//! PLAN-014 T-06：devtools/typed-component 宿主责任（devtools 面板、render_dynamic_view、ComponentIced/TickWrap/DevToolsState/Wrapper、NativePixelsHost）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::host::*;
use super::media::*;
use super::style_widgets::*;
use super::subscriptions::*;
use iced::widget::column;

/// PLAN-646: marquee 框选蒙层绘制程序——半透明填充 + 边框，纯绘制不捕获
/// 事件（broker_surface DrawListPainter 同款 canvas::Program 形态）。
pub(super) struct MarqueePainter {
    pub(super) rect: crate::ui::debug::Rect,
}

impl iced::widget::canvas::Program<IcedMessage> for MarqueePainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        // 蒙层铺满整窗（层坐标 = 窗口坐标，原点一致），矩形按 marquee 画。
        let rect = iced::Rectangle::new(
            iced::Point::new(self.rect.x, self.rect.y),
            iced::Size::new(self.rect.width.max(1.0), self.rect.height.max(1.0)),
        );
        frame.fill_rectangle(
            rect.position(),
            rect.size(),
            iced::Color::from_rgba(0.30, 0.50, 0.90, 0.12),
        );
        frame.stroke(
            &Path::rectangle(rect.position(), rect.size()),
            Stroke::default()
                .with_color(iced::Color::from_rgba(0.30, 0.50, 0.90, 0.9))
                .with_width(1.5),
        );
        vec![frame.into_geometry()]
    }
}

/// Render the DevTools panel on the right side of the window.
///
/// Plan 309 续篇: 元素树 (VTree) 与检视 (面包屑 + 子标签) 合并为同屏分屏 ——
/// 左树点任意 VNode 即设 `selected_vnode`，右侧检视随之更新；两者始终同屏，
/// 不再有互斥 tab。控制台保留为独立整宽模式（点「控制台」按钮切换）。
pub(super) fn render_devtools_panel(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let current_tab = *state.app.devtools.devtools_tab.borrow();

    // Header: [🔍 检视] [控制台] ... [×]
    let inspect_active = *state.app.devtools.inspect_mode.borrow();
    let tab_inspect = container(mouse_area(text("🔍 检视").size(11)).on_press(IcedMessage {
        widget: String::new(),
        event: "__toggle_inspect".to_string(),
        input_value: None,
    }))
    .style(tab_style_fn(inspect_active))
    .padding(iced::Padding::new(4.0));

    let tab_console = container(mouse_area(text("控制台").size(11)).on_press(IcedMessage {
        widget: String::new(),
        event: "__tab_console".to_string(),
        input_value: None,
    }))
    .style(tab_style_fn(current_tab == DevToolsTab::Console))
    .padding(iced::Padding::new(4.0));

    // PLAN-646: Select（采集）标签页 chip。
    let tab_select = container(mouse_area(text("采集").size(11)).on_press(IcedMessage {
        widget: String::new(),
        event: "__tab_select".to_string(),
        input_value: None,
    }))
    .style(tab_style_fn(current_tab == DevToolsTab::Select))
    .padding(iced::Padding::new(4.0));

    let close_btn = container(
        mouse_area(
            text("✕")
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .on_press(IcedMessage {
            widget: String::new(),
            event: "__close_devtools".to_string(),
            input_value: None,
        }),
    )
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.95, 0.95, 0.95,
        ))),
        border: iced::Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding(iced::Padding::new(4.0));

    let tab_bar = row![tab_inspect, tab_console, tab_select]
        .spacing(2)
        .width(iced::Length::Fill);
    let header = row![tab_bar, close_btn]
        .spacing(4)
        .width(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    // Content: split view (Inspect) or full-width console.
    let panel_width = *state.app.devtools.devtools_panel_width.borrow();
    let content: iced::Element<'static, IcedMessage> = match current_tab {
        DevToolsTab::Inspect => {
            // Plan 309 续篇: 同屏分屏 [Tree | divider | Inspector]。分隔栏
            // 用 mouse_area::on_press 设拖拽标志，实际位移由窗口级
            // `__mouse_moved` 订阅按绝对坐标计算（与外层 DevTools 分隔栏同
            // 一套机制；pane_grid 的组件借用 State 与本渲染器返回的
            // `Element<'static>` 契约不兼容，故手写分屏）。
            let ratio = (*state.app.devtools.inspector_split_ratio.borrow()).clamp(0.1, 0.9);
            let is_dragging = *state.app.devtools.dragging_inner_divider.borrow();
            let divider_bg = if is_dragging {
                iced::Color::from_rgb(0.3, 0.5, 0.9) // blue while dragging
            } else {
                iced::Color::from_rgb(0.82, 0.82, 0.82) // gray normally
            };
            let tree_pane = scrollable(render_elements_tab(state))
                .id(state.app.devtools.elements_scroll_id.clone())
                .width(iced::Length::FillPortion((ratio * 1000.0) as u16))
                .height(iced::Length::Fill);
            let inspector_pane = scrollable(render_inspector_tab(state))
                .id(state.app.devtools.inspector_scroll_id.clone())
                .width(iced::Length::FillPortion(((1.0 - ratio) * 1000.0) as u16))
                .height(iced::Length::Fill);
            let inner_divider = mouse_area(
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
                event: "__inner_divider_press".to_string(),
                input_value: None,
            });
            row![tree_pane, inner_divider, inspector_pane]
                .spacing(0)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        }
        DevToolsTab::Console => container(
            scrollable(render_console_tab(state))
                .id(state.app.devtools.inspector_scroll_id.clone())
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into(),
        // PLAN-646: Select（采集）标签页——框选结果信封三视图 + 复制。
        DevToolsTab::Select => container(
            scrollable(render_select_tab(state))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into(),
    };

    let panel_col = column![header, content]
        .spacing(4)
        .width(panel_width)
        .height(iced::Length::Fill);

    container(panel_col)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.98, 0.98, 0.98,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding(iced::Padding::new(6.0))
        .width(panel_width)
        .height(iced::Length::Fill)
        .into()
}

/// PLAN-646: Select（采集）标签页内容——信封头、三视图切换 chips、复制
/// 按钮（带反馈），以及当前视图全文（Auto 切片 / JSON / Atom）。
fn render_select_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let fmt = *state.app.devtools.select_view.borrow();
    let feedback = *state.app.devtools.select_copy_feedback.borrow();
    let feedback_el = match feedback {
        Some(true) => text("已复制 ✓")
            .size(11)
            .color(iced::Color::from_rgb(0.16, 0.55, 0.30)),
        Some(false) => text("复制失败 ✕")
            .size(11)
            .color(iced::Color::from_rgb(0.75, 0.25, 0.25)),
        None => text("").size(11),
    };

    let mut chips = row![].spacing(4);
    for f in [
        crate::ui::selection::SelectionFormat::Auto,
        crate::ui::selection::SelectionFormat::Json,
        crate::ui::selection::SelectionFormat::Atom,
    ] {
        let chip = container(mouse_area(text(f.label()).size(11)).on_press(IcedMessage {
            widget: String::new(),
            event: format!("{}{}", SELECT_VIEW_PREFIX, f.label()),
            input_value: None,
        }))
        .style(tab_style_fn(fmt == f))
        .padding(iced::Padding::new(3.0));
        chips = chips.push(chip);
    }
    let copy_btn = container(mouse_area(text("复制").size(11)).on_press(IcedMessage {
        widget: String::new(),
        event: "__select_copy".to_string(),
        input_value: None,
    }))
    .style(tab_style_fn(false))
    .padding(iced::Padding::new(3.0));

    let body: iced::Element<'static, IcedMessage> =
        match state.app.devtools.selection_result.borrow().as_ref() {
            Some(r) => {
                let header = format!(
                    "surface={} app={} rect=({:.0},{:.0},{:.0},{:.0}) nodes={}",
                    r.surface,
                    r.app,
                    r.rect.0,
                    r.rect.1,
                    r.rect.2,
                    r.rect.3,
                    r.nodes.len()
                );
                let content = r.render(fmt);
                // 正文显式深灰——面板浅色底为硬编码，text 缺省色随 app 主题
                // （深色主题=白字）会白底白字隐身（复审实机截图定罪）。
                column![
                    text(header)
                        .size(11)
                        .color(iced::Color::from_rgb(0.35, 0.35, 0.4)),
                    text(content)
                        .size(11)
                        .color(iced::Color::from_rgb(0.15, 0.15, 0.18)),
                ]
                .spacing(6)
                .into()
            }
            None => {
                text("Alt+拖拽框选任意区域采集组件源码；或经 MCP autoui_select_rect 程序化采集。")
                    .size(11)
                    .color(iced::Color::from_rgb(0.45, 0.45, 0.5))
                    .into()
            }
        };

    column![
        row![chips, copy_btn, feedback_el]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        body,
    ]
    .spacing(6)
    .width(iced::Length::Fill)
    .into()
}

fn tab_style_fn(active: bool) -> Box<dyn Fn(&iced::Theme) -> container::Style> {
    Box::new(move |_: &iced::Theme| {
        if active {
            container::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.5, 0.9),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(iced::Color::from_rgb(0.2, 0.2, 0.2)),
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.93, 0.93, 0.93,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                ..Default::default()
            }
        }
    })
}

/// Render the Properties tab: show selected element's style properties.
/// Render the Elements tab: component tree visualization.
fn render_elements_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    // Plan 307 Task 14: the left tree now reads from the live VTree (the runtime
    // DOM) instead of the legacy DebugTreeNode / component_tree. The old path is
    // kept (Task 19/20 removes it); render_tree_into simply isn't called here.
    let vtree = state.app.live_vtree.borrow();
    let has_root = vtree.as_ref().and_then(|t| t.root()).is_some();
    if has_root {
        // Clone the tree out so we don't hold the RefCell borrow while building rows.
        let tree = vtree.clone().expect("checked root above");
        let selected = state.app.devtools.selected_vnode.borrow().clone();
        let hovered = state.app.devtools.hovered_vnode.borrow().clone();
        let mut rows: Vec<iced::Element<'static, IcedMessage>> = Vec::new();
        if let Some(root) = tree.root() {
            render_vtree_into(&tree, root, 0, &selected, &hovered, &mut rows);
        }
        drop(vtree);
        let mut col = column![].spacing(1);
        for row in rows {
            col = col.push(row);
        }
        col.into()
    } else {
        drop(vtree);
        column![
            text("组件树不可用")
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text("开启 Debug 模式后显示")
                .size(10)
                .color(iced::Color::from_rgb(0.4, 0.4, 0.4)),
        ]
        .spacing(4)
        .into()
    }
}

/// Render a per-kind summary string for a VNode's props (Plan 307 Task 14).
fn vnode_summary(node: &crate::ui::vnode::VNode) -> String {
    use crate::ui::vnode::{VNodeKind, VNodeProps};
    let child_count = node.children.len();
    match (&node.kind, &node.props) {
        (VNodeKind::Text, VNodeProps::Text { content, .. }) => {
            let snippet: String = content.chars().take(20).collect();
            if content.chars().count() > 20 {
                format!("\"{}…\"", snippet)
            } else {
                format!("\"{}\"", snippet)
            }
        }
        (VNodeKind::Button, VNodeProps::Button { label, .. }) => {
            let snippet: String = label.chars().take(20).collect();
            format!("[{}]", snippet)
        }
        (VNodeKind::Input, VNodeProps::Input { placeholder, .. }) => {
            format!("placeholder=\"{}\"", placeholder)
        }
        (VNodeKind::Textarea, VNodeProps::Textarea { placeholder, .. }) => {
            format!("placeholder=\"{}\"", placeholder)
        }
        (VNodeKind::Checkbox, VNodeProps::Checkbox { label, is_checked }) => {
            format!("{}={}", label, if *is_checked { "✓" } else { "✗" })
        }
        (VNodeKind::Radio, VNodeProps::Radio { label, is_selected }) => {
            format!("{}={}", label, if *is_selected { "✓" } else { "✗" })
        }
        (
            VNodeKind::Select,
            VNodeProps::Select {
                options,
                selected_index,
            },
        ) => {
            format!("{} opts, sel {:?}", options.len(), selected_index)
        }
        (VNodeKind::Slider, VNodeProps::Slider { value, .. }) => {
            format!("value={:.2}", value)
        }
        (VNodeKind::ProgressBar, VNodeProps::ProgressBar { progress }) => {
            format!("{:.0}%", progress * 100.0)
        }
        // Containers: show child count
        (_, _) if child_count > 0 => format!("({} children)", child_count),
        _ => String::new(),
    }
}

/// Recursively render live VTree nodes into a flat column of clickable rows
/// (Plan 307 Task 14). Modeled on the legacy `render_tree_into` row style —
/// all nodes start expanded (no collapse state yet).
fn render_vtree_into(
    tree: &crate::ui::vnode::VTree,
    node: &crate::ui::vnode::VNode,
    depth: usize,
    selected: &Option<crate::ui::vnode::VNodeId>,
    hovered: &Option<crate::ui::vnode::VNodeId>,
    rows: &mut Vec<iced::Element<'static, IcedMessage>>,
) {
    let indent = "  ".repeat(depth);
    let is_selected = *selected == Some(node.id);
    let is_hovered = *hovered == Some(node.id);

    let has_children = !node.children.is_empty();
    let prefix = if has_children { "▼ " } else { "  " };
    let summary = vnode_summary(node);
    let label = if summary.is_empty() {
        format!("{}{}{}", indent, prefix, node.kind)
    } else {
        format!("{}{}{} {}", indent, prefix, node.kind, summary)
    };

    let text_color = if is_selected {
        iced::Color::from_rgb(0.85, 0.4, 0.1)
    } else if is_hovered {
        iced::Color::from_rgb(0.3, 0.55, 0.3)
    } else if has_children {
        iced::Color::from_rgb(0.2, 0.4, 0.7)
    } else {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    };

    let click_area = mouse_area(
        container(text(label).size(10).color(text_color))
            .style(move |_: &iced::Theme| {
                if is_selected {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.95, 0.85, 0.7, 0.6,
                        ))),
                        ..Default::default()
                    }
                } else if is_hovered {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.8, 0.9, 0.8, 0.4,
                        ))),
                        ..Default::default()
                    }
                } else {
                    container::Style::default()
                }
            })
            .padding(iced::Padding::new(2.0)),
    )
    .on_press(IcedMessage {
        widget: String::new(),
        event: format!("{}{}", DEBUG_SELECT_VNODE_PREFIX, node.id.as_u64()),
        input_value: None,
    });

    rows.push(click_area.into());

    if let Some(children) = tree.children(node.id) {
        for child in children {
            render_vtree_into(tree, child, depth + 1, selected, hovered, rows);
        }
    }
}

/// AutoLang keywords for syntax highlighting.
const AUTO_KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "var",
    "const",
    "if",
    "else",
    "for",
    "loop",
    "in",
    "break",
    "return",
    "type",
    "enum",
    "use",
    "pub",
    "mut",
    "static",
    "true",
    "false",
    "is",
    "Some",
    "None",
    "Ok",
    "Err",
    "match",
    "where",
    // UI widget tags
    "col",
    "row",
    "text",
    "button",
    "input",
    "container",
    "scroll",
    "checkbox",
    "radio",
    "select",
    "slider",
    "image",
    "link",
    "list",
    "tab",
    "tabs",
    "sidebar",
    "accordion",
    "nav",
    "textarea",
    "progress",
    "code_editor",
];

/// Pure tokenization: returns (text, color) pairs for one source line.
/// No iced widget creation — just data, suitable for caching.
fn tokenize_line(line: &str) -> Vec<(String, iced::Color)> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Comment: //
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            let comment: String = chars[i..].iter().collect();
            spans.push((comment, iced::Color::from_rgb(0.5, 0.55, 0.5)));
            break;
        }
        // String literal: "..."
        if chars[i] == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                } // skip escaped char
                i += 1;
            }
            if i < len {
                i += 1;
            } // closing quote
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.16, 0.6, 0.26)));
            continue;
        }
        // F-string: f"..."
        if chars[i] == 'f' && i + 1 < len && chars[i + 1] == '"' {
            let start = i;
            i += 2;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < len {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.16, 0.55, 0.35)));
            continue;
        }
        // Number
        if chars[i].is_ascii_digit()
            || (chars[i] == '-' && i + 1 < len && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            if chars[i] == '-' {
                i += 1;
            }
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.8, 0.4, 0.1)));
            continue;
        }
        // Identifier or keyword
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let color = if AUTO_KEYWORDS.contains(&word.as_str()) {
                iced::Color::from_rgb(0.15, 0.3, 0.75) // keyword: blue
            } else if word.starts_with(char::is_uppercase) {
                iced::Color::from_rgb(0.5, 0.15, 0.55) // type: purple
            } else {
                iced::Color::from_rgb(0.3, 0.3, 0.3) // default: dark grey
            };
            spans.push((word, color));
            continue;
        }
        // Operators and punctuation
        let ch = chars[i];
        i += 1;
        let color = match ch {
            ':' | '=' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | '.' | '|' | '#' | '@' => {
                iced::Color::from_rgb(0.45, 0.45, 0.45)
            }
            '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' | '&' | '^' => {
                iced::Color::from_rgb(0.55, 0.35, 0.15)
            }
            _ => iced::Color::from_rgb(0.3, 0.3, 0.3),
        };
        spans.push((ch.to_string(), color));
    }
    spans
}

/// Build cached syntax highlighting for all lines in a source file.
/// Called once when source is loaded/changed, reused every frame.
pub(super) fn build_highlight_cache(source: &str) -> Vec<Vec<(String, iced::Color)>> {
    source.lines().map(|line| tokenize_line(line)).collect()
}

/// Render the Inspector tab: source code + properties, stacked vertically.
fn render_inspector_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    // Plan 307 Task 15: the right panel is rebuilt around the VNodeId-based
    // selection. Structure: [breadcrumb] › [sub-tab row] › [active sub-tab body].
    //
    // All prior source-code display logic now lives in
    // `render_inspector_source_section` (preserved for Task 16's Source tab /
    // Task 19-20 cleanup); it is intentionally left callable-but-unused here.

    let mut col = column![].spacing(6);

    // --- Breadcrumb: root › … › selected (clickable ancestors) ---
    col = col.push(render_inspector_breadcrumb(state));

    // --- Inner sub-tab row: 检视 | AutoUI | 源码 ---
    col = col.push(render_inspector_subtab_row(state));

    // --- Active sub-tab body ---
    let subtab = *state.app.devtools.inspector_subtab.borrow();
    let body = match subtab {
        InspectorSubTab::Inspect => render_inspector_inspect_tab(state),
        InspectorSubTab::AutoUI => render_inspector_autoui_tab(state),
        InspectorSubTab::Source => render_inspector_source_tab(state),
    };
    col = col.push(body);

    col.into()
}

/// Render the breadcrumb from the selected VNode up to root as clickable chips
/// (Plan 307 Task 15). Each ancestor chip click re-selects that node via the
/// existing `__select_vnode_<u64>` message from Task 14.
///
/// Reads `live_vtree` and walks the `parent` chain, cloning the tree out first
/// so no RefCell borrow is held across the closure-driven widget construction.
fn render_inspector_breadcrumb(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let vtree = state.app.live_vtree.borrow().clone();
    let selected = state.app.devtools.selected_vnode.borrow().clone();

    let (tree, sel_id) = match (vtree, selected) {
        (Some(tree), Some(id)) => (tree, id),
        // No live tree or no selection: show the empty-state prompt.
        _ => {
            return column![
                text("无选中元素")
                    .size(11)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                text("点击元素以查看")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(2)
            .into();
        }
    };

    // Walk parent chain: selected → … → root, then reverse for display order.
    let mut chain: Vec<crate::ui::vnode::VNodeId> = Vec::new();
    let mut cursor = Some(sel_id);
    // Guard against cycles / runaway walks with a sane depth cap.
    for _ in 0..256 {
        let Some(id) = cursor else { break };
        let Some(node) = tree.get(id) else { break };
        chain.push(id);
        cursor = node.parent;
    }
    chain.reverse(); // root first

    // Build the chip row: root › col › row ▸ [selected]
    let mut row = row![].spacing(2).align_y(iced::Alignment::Center);
    let total = chain.len();
    for (idx, &id) in chain.iter().enumerate() {
        let is_last = idx == total - 1;
        let label_text = match tree.get(id) {
            Some(node) => {
                // Prefer the debug label; fall back to the kind.
                if !node.label.is_empty() {
                    node.label.clone()
                } else {
                    format!("{:?}", node.kind)
                }
            }
            None => "?".to_string(),
        };

        let chip = if is_last {
            // Selected (leaf): emphasize with ▸ and a tinted background.
            container(
                mouse_area(
                    container(text(format!("▸ {}", label_text)).size(10))
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.95, 0.85, 0.7, 0.7,
                            ))),
                            border: iced::Border {
                                radius: 3.0.into(),
                                color: iced::Color::from_rgb(0.8, 0.6, 0.3),
                                width: 1.0,
                            },
                            ..Default::default()
                        })
                        .padding(iced::Padding::new(2.0)),
                )
                .on_press(select_vnode_message(id)),
            )
            .padding(iced::Padding::new(0.0))
        } else {
            // Clickable ancestor.
            container(
                mouse_area(
                    container(
                        text(label_text)
                            .size(10)
                            .color(iced::Color::from_rgb(0.2, 0.4, 0.7)),
                    )
                    .padding(iced::Padding::new(2.0)),
                )
                .on_press(select_vnode_message(id)),
            )
        };
        row = row.push(chip);

        if !is_last {
            row = row.push(
                text("›")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
    }

    row.into()
}

/// The 检视 sub-tab body (Plan 307 续篇 IV): a single scrollable column (the
/// parent `scrollable` at the panel level handles overflow) of three
/// collapsible sections — Box Model, Computed, Properties — each reusing the
/// existing per-section render fn as its body.
fn render_inspector_inspect_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let secs = *state.app.devtools.inspector_sections.borrow();
    let mut col = column![].spacing(6);

    col = col.push(render_collapsible_section(
        "盒模型 Box Model",
        secs.box_collapsed,
        "box",
        render_inspector_layout_tab(state),
    ));
    col = col.push(render_collapsible_section(
        "Computed",
        secs.computed_collapsed,
        "computed",
        render_inspector_computed_tab(state),
    ));
    col = col.push(render_collapsible_section(
        "Properties",
        secs.props_collapsed,
        "props",
        render_inspector_props_tab(state),
    ));

    col.into()
}

/// One collapsible section: a clickable header (▸/▾ + title) followed by the
/// body when expanded. The header click sends `__inspector_section_<tail>`,
/// parsed in `update()` to toggle the matching `*_collapsed` bool.
fn render_collapsible_section(
    title: &'static str,
    collapsed: bool,
    tail: &str,
    body: iced::Element<'static, IcedMessage>,
) -> iced::Element<'static, IcedMessage> {
    let marker = if collapsed { "▸" } else { "▾" };
    let header = mouse_area(
        row![
            text(marker).size(10),
            text(title)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.8)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .on_press(IcedMessage {
        widget: String::new(),
        event: format!("{}{}", DEBUG_INSPECTOR_SECTION_PREFIX, tail),
        input_value: None,
    });

    let mut col = column![]
        .spacing(3)
        .push(container(header).padding([2.0, 4.0]));
    if !collapsed {
        col = col.push(body);
    }
    col.into()
}

/// Build the inner sub-tab chip row (Plan 307 Task 15). Clicking a chip sends
/// `__inspector_subtab_<Variant>`, parsed in `update()`.
fn render_inspector_subtab_row(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let active = *state.app.devtools.inspector_subtab.borrow();
    let variants = [
        InspectorSubTab::Inspect,
        InspectorSubTab::AutoUI,
        InspectorSubTab::Source,
    ];

    let mut row = row![].spacing(2);
    for v in variants {
        let is_active = v == active;
        let chip = container(mouse_area(text(v.label()).size(10)).on_press(IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_INSPECTOR_SUBTAB_PREFIX, v.label()),
            input_value: None,
        }))
        .style(tab_style_fn(is_active))
        .padding(iced::Padding::new(3.0));
        row = row.push(chip);
    }
    row.into()
}

/// Layout sub-tab: box model visualization for the selected node
/// (Plan 307 Task 15).
///
/// Reads `live_cache` (bounds/box_model). Falls back to "(布局中…)" when the
/// node isn't laid out yet or has no cache entry, per design §6.1.
fn render_inspector_layout_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let selected = state.app.devtools.selected_vnode.borrow().clone();
    let Some(sel_id) = selected else {
        return placeholder_panel("无选中元素");
    };

    let cache = state.app.live_cache.borrow().clone();
    let Some(cache) = cache else {
        // Not in debug mode (no cache built this frame).
        return layout_pending_panel();
    };

    let Some(computed) = cache.get(sel_id) else {
        // Selected node has no entry in the cache yet.
        return layout_pending_panel();
    };

    // Need a layout: box_model is preferred, else bounds alone.
    let bm = match (&computed.box_model, &computed.bounds) {
        (Some(bm), _) => bm.clone(),
        (None, Some(b)) => crate::ui::debug::BoxModel::from_bounds(*b),
        (None, None) => return layout_pending_panel(),
    };

    let mut col = column![].spacing(4);

    // Chrome-style nested box-model diagram (Plan 309 Phase 3.4). Each layer's
    // drawn inset is capped so oversized margins stay within the panel; numeric
    // rows below remain truthful.
    col = col.push(render_box_model_diagram(&bm));

    // Content rect: x,y  W×H
    let content = bm.content;
    col = col.push(
        row![
            text("Content:")
                .size(10)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!(
                "x={:.0} y={:.0}   {:.0} × {:.0}",
                content.x, content.y, content.width, content.height
            ))
            .size(10)
            .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
        ]
        .spacing(6),
    );

    // Padding (declared value; currently zero from Task 13 — label is
    // forward-looking per the design).
    col = col.push(layout_inset_row("Padding", &bm.padding, Some("(声明值)")));
    // Margin.
    col = col.push(layout_inset_row("Margin", &bm.margin, None));

    // Border box + margin box summaries (derived).
    let bb = bm.border_box();
    let mb = bm.margin_box();
    col = col.push(
        row![
            text("Border box:")
                .size(9)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!("{:.0} × {:.0}", bb.width, bb.height))
                .size(9)
                .color(iced::Color::from_rgb(0.35, 0.35, 0.35)),
        ]
        .spacing(6),
    );
    col = col.push(
        row![
            text("Margin box:")
                .size(9)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!("{:.0} × {:.0}", mb.width, mb.height))
                .size(9)
                .color(iced::Color::from_rgb(0.35, 0.35, 0.35)),
        ]
        .spacing(6),
    );

    col.into()
}

/// One labeled padding/margin row: `Label:  t / r / b / l  [annotation]`.
fn layout_inset_row(
    label: &str,
    ei: &crate::ui::debug::EdgeInsets,
    annotation: Option<&str>,
) -> iced::Element<'static, IcedMessage> {
    let mut row = row![
        text(format!("{}:", label))
            .size(10)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text(format!(
            "{:.0} / {:.0} / {:.0} / {:.0}",
            ei.top, ei.right, ei.bottom, ei.left
        ))
        .size(10)
        .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
    ]
    .spacing(6);
    if let Some(note) = annotation {
        // Own the string so the returned Element can be 'static.
        row = row.push(
            text(note.to_string())
                .size(9)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        );
    }
    row.into()
}

/// Nested box-model diagram (Plan 309 Phase 3.4): margin → border → padding →
/// content, each layer a colored `container` wrapping the next. Drawn insets
/// are capped (`cap`) so large declared margins fit the narrow inspector panel;
/// the numeric rows in the Layout tab hold the exact values.
fn render_box_model_diagram<M: Clone + 'static>(
    bm: &crate::ui::debug::BoxModel,
) -> iced::Element<'static, M> {
    use iced::widget::container;
    // Cap each side's drawn inset at 28px for display only.
    let cap = |v: f32| v.min(28.0);
    let pad = |ei: &crate::ui::debug::EdgeInsets| iced::Padding {
        top: cap(ei.top),
        right: cap(ei.right),
        bottom: cap(ei.bottom),
        left: cap(ei.left),
    };

    // Innermost: content (light blue), labelled with its measured W×H.
    let content_label = text(format!(
        "{} × {}",
        bm.content.width.round() as i32,
        bm.content.height.round() as i32
    ))
    .size(9)
    .color(iced::Color::from_rgb(0.1, 0.2, 0.5));
    let content_layer = container(content_label)
        .padding(2.0)
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.91, 0.94, 0.99, 1.0,
            ))),
            ..Default::default()
        });

    // Padding layer (pale yellow) wraps content.
    let padding_layer = container(content_layer)
        .padding(pad(&bm.padding))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                1.0, 0.98, 0.85, 1.0,
            ))),
            ..Default::default()
        });

    // Border layer (dark line) wraps padding.
    let border_layer = container(padding_layer)
        .padding(pad(&bm.border))
        .style(move |_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.95, 0.95, 0.95, 1.0,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Margin layer (transparent w/ dashed-look label) wraps border.
    let margin_layer = container(border_layer)
        .padding(pad(&bm.margin))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.85, 0.85, 0.85, 0.25,
            ))),
            ..Default::default()
        });

    // Legend strip above the nested diagram.
    let legend = row![
        text("margin")
            .size(8)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text("•").size(8),
        text("border")
            .size(8)
            .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
        text("•").size(8),
        text("padding")
            .size(8)
            .color(iced::Color::from_rgb(0.7, 0.6, 0.2)),
        text("•").size(8),
        text("content")
            .size(8)
            .color(iced::Color::from_rgb(0.1, 0.2, 0.5)),
    ]
    .spacing(4);

    column![legend, margin_layer].spacing(4).into()
}

/// "布局中…" placeholder for nodes not yet laid out (design §6.1).
///
/// Generic over the message type (Plan 311): pure text, shared by VM and
/// rust-mode panels.
fn layout_pending_panel<M: Clone + 'static>() -> iced::Element<'static, M> {
    text("(布局中…)")
        .size(11)
        .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
        .into()
}

/// Lazily load the component source + derived indexes into the session's
/// App state (Plan 309 Phase 4.1; T4c 起经共享拆借视图访问). Shared by the element-select (`__select_`) and
/// VNode-select (`__select_vnode_`) handlers so the Source sub-tab can render
/// the source listing regardless of which selection path opened it. No-op once
/// already loaded.
pub(super) fn ensure_source_loaded(state: crate::ui::session::SessionViewRef) {
    if state.app.source_code.borrow().is_some() {
        return;
    }
    let Some(path) = state.component.source_path() else {
        return;
    };
    let Ok(code) = std::fs::read_to_string(path) else {
        return;
    };
    // Compute line byte offsets for span→line mapping.
    let mut offsets = vec![0usize];
    for (i, ch) in code.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    *state.app.source_line_offsets.borrow_mut() = offsets;
    *state.app.devtools.cached_highlighted.borrow_mut() = Some(build_highlight_cache(&code));
    // Build line → AuraNodeId index for source-click → component-highlight.
    let span_map = state.component.span_map().clone();
    *state.app.line_to_aura_ids.borrow_mut() = build_line_to_aura_ids(&span_map, &code);
    *state.app.source_code.borrow_mut() = Some(code);
}

/// Helper: clone the selected VNode out of `live_vtree`, or return a grey
/// placeholder Element if there is no tree / no selection.
fn with_selected_vnode<F>(
    state: crate::ui::session::SessionViewRef,
    on_missing: &str,
    f: F,
) -> iced::Element<'static, IcedMessage>
where
    F: FnOnce(&crate::ui::vnode::VNode) -> iced::Element<'static, IcedMessage>,
{
    let vtree = state.app.live_vtree.borrow().clone();
    let selected = state.app.devtools.selected_vnode.borrow().clone();
    match (vtree, selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => f(node),
            None => placeholder_panel(on_missing),
        },
        _ => placeholder_panel(on_missing),
    }
}

/// One `key: value` row, used by the Props / Computed tabs.
///
/// Generic over the message type (Plan 311): non-interactive text/row only, so
/// the VM inspector (M = `IcedMessage`) and the rust-mode DevTools panel
/// (M = `WrapperMsg<C>`) share the same helper. M resolves by inference at each
/// call site.
fn kv_row<M: Clone + 'static>(key: &str, value: String) -> iced::Element<'static, M> {
    row![
        text(format!("{}:", key))
            .size(10)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text(value)
            .size(10)
            .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
    ]
    .spacing(6)
    .into()
}

/// Props sub-tab: render the selected VNode's `VNodeProps` fields plus `kind`
/// and `path` (Plan 307 Task 16).
///
/// Data source: `live_vtree` only (the VNode carries its own props). No probe /
/// cache dependency, so it always works whenever a node is selected.
fn render_inspector_props_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let mut col = column![].spacing(3);

        col = col.push(kv_row("kind", format!("{:?}", node.kind)));
        col = col.push(kv_row(
            "path",
            format!(
                "[{}]",
                node.path
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));

        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Empty => {}
            VNodeProps::Text { content, .. } => col = col.push(kv_row("content", content.clone())),
            VNodeProps::Button { label, .. } => col = col.push(kv_row("label", label.clone())),
            VNodeProps::Input {
                placeholder,
                value,
                password,
            } => {
                col = col.push(kv_row("placeholder", placeholder.clone()));
                col = col.push(kv_row("value", value.clone()));
                col = col.push(kv_row("password", password.to_string()));
            }
            VNodeProps::Textarea { placeholder, value } => {
                col = col.push(kv_row("placeholder", placeholder.clone()));
                col = col.push(kv_row("value", value.clone()));
            }
            VNodeProps::Checkbox { label, is_checked } => {
                col = col.push(kv_row("label", label.clone()));
                col = col.push(kv_row("is_checked", is_checked.to_string()));
            }
            VNodeProps::Radio { label, is_selected } => {
                col = col.push(kv_row("label", label.clone()));
                col = col.push(kv_row("is_selected", is_selected.to_string()));
            }
            VNodeProps::Select {
                options,
                selected_index,
            } => {
                col = col.push(kv_row("options", format!("[{}]", options.join(", "))));
                col = col.push(kv_row("selected_index", format!("{:?}", selected_index)));
            }
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row("padding", padding.to_string()));
                col = col.push(kv_row("center_x", center_x.to_string()));
                col = col.push(kv_row("center_y", center_y.to_string()));
            }
            VNodeProps::Scrollable { .. } => {}
            VNodeProps::Slider {
                min,
                max,
                value,
                step,
            } => {
                col = col.push(kv_row("min", format!("{}", min)));
                col = col.push(kv_row("max", format!("{}", max)));
                col = col.push(kv_row("value", format!("{}", value)));
                col = col.push(kv_row("step", format!("{:?}", step)));
            }
            VNodeProps::ProgressBar { progress } => {
                col = col.push(kv_row("progress", format!("{}", progress)));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
                ..
            } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("col_spacing", col_spacing.to_string()));
            }
        }

        col.into()
    })
}

/// AutoUI sub-tab: `state_bindings` / `for_context` / `events` for the selected
/// node (Plan 307 Task 16).
///
/// Path-scheme caveat: the probe is keyed by build-time (AuraNode-structural)
/// path, while `VNode.path` is the View-structural path. They coincide for
/// non-loop nodes, so we look the probe up via `snapshot().get(&node.path)`.
/// For loop-body nodes the schemes diverge and the lookup misses — we degrade
/// gracefully to a grey hint rather than panicking (design §6.1).
fn render_inspector_autoui_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let probe = state.app.live_probe.borrow().clone();
        let Some(probe) = probe else {
            return placeholder_panel("(AutoUI 探针未启用)");
        };

        let Some(entry) = probe.snapshot().get(&node.path) else {
            // Path-scheme divergence (e.g. for-loop body) or genuinely no
            // AutoUI metadata for this node — degrade gracefully.
            return placeholder_panel("(本节点无 AutoUI 元数据)");
        };

        let mut col = column![].spacing(3);

        if !entry.state_bindings.is_empty() {
            col = col.push(
                text("状态绑定")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            for sb in &entry.state_bindings {
                let val = if sb.current_value.is_empty() {
                    "<unresolved>".to_string()
                } else {
                    sb.current_value.clone()
                };
                col = col.push(kv_row(&sb.expr, val));
            }
        }

        if let Some(fc) = &entry.for_context {
            col = col.push(
                text("循环上下文")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            col = col.push(kv_row(
                "for",
                format!(
                    "{}={}, i={}",
                    fc.var,
                    fc.value_repr,
                    match fc.index {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    }
                ),
            ));
        }

        if !entry.events.is_empty() {
            col = col.push(
                text("事件")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            for ev in &entry.events {
                col = col.push(kv_row(&ev.event, ev.handler.clone()));
            }
        }

        if entry.state_bindings.is_empty() && entry.for_context.is_none() && entry.events.is_empty()
        {
            // Entry exists but is empty.
            return placeholder_panel("(本节点无 AutoUI 元数据)");
        }

        col.into()
    })
}

/// Source sub-tab (Plan 307 Task 16; real viewer wired in Plan 309 Phase 4.1).
///
/// Resolves the selected VNode's `source_span` → a 0-based half-open
/// `(start_line, end_line)` highlight range, then delegates to
/// [`render_source_viewer`] for the syntax-highlighted listing. Clicking a
/// line that has an associated AuraNodeId (handled by `SRC_CLICK_PREFIX`)
/// selects the corresponding element — bidirectional navigation.
fn render_inspector_source_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        // Resolve the span → a 0-based half-open (start, end) line range.
        let highlight_range = node.source_span.map(|span| {
            let line_offsets = state.app.source_line_offsets.borrow();
            let start_line = line_offsets
                .partition_point(|&pos| pos <= span.offset)
                .saturating_sub(1);
            let end_line = line_offsets.partition_point(|&pos| pos < span.offset + span.len);
            (start_line, end_line.max(start_line))
        });

        let basename = state
            .component
            .source_path()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "source".to_string());

        let header_line = highlight_range
            .map(|(s, _)| format!("{}:{}", basename, s + 1))
            .unwrap_or_else(|| basename);

        let mut col = column![].spacing(4);
        col = col.push(
            text(header_line)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.7)),
        );
        col = col.push(render_source_viewer(state, highlight_range));
        col.into()
    })
}

/// Reusable source-code listing for the Source sub-tab (Plan 309 Phase 4.1).
///
/// Renders the cached component source with syntax highlighting; highlights
/// `highlight_range` (0-based half-open `(start_line, end_line)`; `None` = no
/// highlight); wraps lines that have an associated AuraNodeId in a
/// `mouse_area` emitting `SRC_CLICK_PREFIX<line>` so a line click selects the
/// element (bidirectional with element/tree selection).
fn render_source_viewer(
    state: crate::ui::session::SessionViewRef,
    highlight_range: Option<(usize, usize)>,
) -> iced::Element<'static, IcedMessage> {
    let source = state.app.source_code.borrow().clone();
    let path_display = state
        .component
        .source_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut col = column![].spacing(2);

    match source {
        Some(code) => {
            col = col.push(
                text(path_display)
                    .size(9)
                    .color(iced::Color::from_rgb(0.4, 0.6, 0.8)),
            );

            let cached = state.app.devtools.cached_highlighted.borrow();
            let all_lines: Vec<&str> = code.lines().collect();
            let total = all_lines.len();
            let line_map = state.app.line_to_aura_ids.borrow();

            for i in 0..total {
                let line_num = format!("{:>4}", i + 1);
                let is_highlighted = highlight_range
                    .map(|(hs, he)| i >= hs && i < he)
                    .unwrap_or(false);
                let has_aura = line_map.contains_key(&i);

                let mut line_row = row![].spacing(0);
                if is_highlighted {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.8, 0.4, 0.1)),
                    );
                } else {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    );
                }

                if let Some(ref cache) = *cached {
                    if let Some(cached_line) = cache.get(i) {
                        for (fragment, color) in cached_line {
                            line_row = line_row.push(text(fragment.clone()).size(10).color(*color));
                        }
                    } else if let Some(line) = all_lines.get(i) {
                        line_row = line_row.push(
                            text(line.to_string())
                                .size(10)
                                .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                        );
                    }
                } else if let Some(line) = all_lines.get(i) {
                    line_row = line_row.push(
                        text(line.to_string())
                            .size(10)
                            .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                    );
                }

                let bg_color = if is_highlighted {
                    iced::Color::from_rgb(1.0, 0.95, 0.85)
                } else if has_aura {
                    iced::Color::from_rgb(0.94, 0.96, 1.0)
                } else {
                    iced::Color::TRANSPARENT
                };

                let line_container = container(line_row.spacing(4))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(1.0))
                    .width(iced::Length::Fill);

                if has_aura {
                    let line_idx = i;
                    let ma = mouse_area(line_container).on_press(IcedMessage {
                        widget: String::new(),
                        event: format!("{}{}", SRC_CLICK_PREFIX, line_idx),
                        input_value: None,
                    });
                    col = col.push(ma);
                } else {
                    col = col.push(line_container);
                }
            }
            drop(line_map);
        }
        None => {
            col = col.push(
                text("(源码未加载)")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
    }

    col.into()
}

/// Computed sub-tab: interim "computed layout" view (Plan 307 Task 16).
///
/// Honest limitation: `VNodeProps` carries no CSS class/style, and
/// `ComputedNode.computed_style`/`raw_class` are not yet populated by the cache
/// builder, so a full CSS computed-style sheet is not possible. We render the
/// layout-relevant props from `VNodeProps` plus the live_cache `bounds` /
/// `box_model` summary, and note that class resolution is pending.
fn render_inspector_computed_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let mut col = column![].spacing(3);

        // --- Layout-relevant props from VNodeProps ---
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row("padding", padding.to_string()));
                col = col.push(kv_row("center_x", center_x.to_string()));
                col = col.push(kv_row("center_y", center_y.to_string()));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
                ..
            } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("col_spacing", col_spacing.to_string()));
            }
            _ => {
                col = col.push(
                    text("(本节点类型无布局计算属性)")
                        .size(10)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                );
            }
        }

        // --- Live bounds / box model + computed style from InspectorCache ---
        let cache = state.app.live_cache.borrow().clone();
        let mut have_computed = false;
        if let Some(cache) = cache {
            if let Some(computed) = cache.get(node.id) {
                if let Some(bm) = &computed.box_model {
                    let c = &bm.content;
                    col = col.push(kv_row(
                        "content",
                        format!("{:.0}×{:.0} @({:.0},{:.0})", c.width, c.height, c.x, c.y),
                    ));
                } else if let Some(b) = &computed.bounds {
                    col = col.push(kv_row(
                        "bounds",
                        format!("{:.0}×{:.0} @({:.0},{:.0})", b.width, b.height, b.x, b.y),
                    ));
                }
                // Plan 309 Phase 2c: raw class + computed style. `raw_class`
                // is the faithful `class="..."` declaration (via BuildProbe);
                // `computed_style` is the parsed props (via debug_style_props).
                if let Some(class_str) = &computed.raw_class {
                    col = col.push(kv_row("class", class_str.clone()));
                    have_computed = true;
                }
                for (k, v) in &computed.computed_style {
                    col = col.push(kv_row(k.as_str(), v.clone()));
                    have_computed = true;
                }
            }
        }

        if !have_computed {
            col = col.push(
                text("(无 computed 样式)")
                    .size(9)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        col.into()
    })
}

/// Generic greyed placeholder body for not-yet-implemented sub-tabs.
///
/// Generic over the message type (Plan 311): pure text, shared by VM and
/// rust-mode panels.
fn placeholder_panel<M: Clone + 'static>(msg: &str) -> iced::Element<'static, M> {
    text(msg.to_string())
        .size(11)
        .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
        .into()
}

/// Construct the `__select_vnode_<u64>` selection message (Task 14 pattern).
fn select_vnode_message(id: crate::ui::vnode::VNodeId) -> IcedMessage {
    IcedMessage {
        widget: String::new(),
        event: format!("{}{}", DEBUG_SELECT_VNODE_PREFIX, id.as_u64()),
        input_value: None,
    }
}

/// Legacy source-code display section (Plan 307 Task 15).
///
/// This is the *previous* body of `render_inspector_tab`, preserved verbatim so
/// Task 16 (Source tab) can reuse it and Task 19/20 can retire it cleanly. It
/// is intentionally not wired into the new right panel yet — keep it here as
/// `#[allow(dead_code)]` until then.
#[allow(dead_code)]
fn render_inspector_source_section(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let selected_id = state.app.devtools.selected_widget.borrow().clone();
    let styles = state.app.devtools.debug_element_styles.borrow();
    let info = selected_id.as_ref().and_then(|id| styles.get(id));

    let mut col = column![].spacing(4);

    // --- Properties section (top) ---
    match info {
        Some(elem_info) => {
            let title = format!(
                "{} #{}",
                elem_info.kind,
                selected_id.as_deref().unwrap_or("?")
            );
            col = col.push(
                text(title)
                    .size(12)
                    .color(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            );
            if !elem_info.props.is_empty() {
                col = col.push(
                    text("样式属性")
                        .size(10)
                        .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
                );
                for (k, v) in &elem_info.props {
                    col = col.push(
                        row![
                            text(format!("{}:", k))
                                .size(11)
                                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                            text(v.clone())
                                .size(11)
                                .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
                        ]
                        .spacing(4),
                    );
                }
            }
        }
        None => {
            col = col.push(
                text("无选中元素")
                    .size(11)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            );
            col = col.push(
                text("点击元素以查看属性和源码")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
    }

    // --- Divider + Source section (bottom) ---
    let source = state.app.source_code.borrow().clone();
    let path_display = state
        .component
        .source_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Determine highlighted line range from selected element's span
    let highlight_range = info.and_then(|elem_info| {
        elem_info.span.and_then(|(offset, len)| {
            let line_offsets = state.app.source_line_offsets.borrow();
            // Find start line (first line_offset <= offset)
            let start_line = line_offsets
                .partition_point(|&pos| pos <= offset)
                .saturating_sub(1);
            // Find end line (first line_offset >= offset + len)
            let end_offset = offset + len;
            let end_line = line_offsets.partition_point(|&pos| pos < end_offset);
            Some((start_line, end_line.max(start_line)))
        })
    });

    // Divider line with "源码" label
    col = col.push(
        container(
            row![text("───── 源码 ─────")
                .size(10)
                .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),]
            .width(iced::Length::Fill)
            .align_y(iced::Alignment::Center),
        )
        .width(iced::Length::Fill)
        .padding(iced::Padding::new(4.0)),
    );

    match source {
        Some(code) => {
            col = col.push(
                text(path_display)
                    .size(9)
                    .color(iced::Color::from_rgb(0.4, 0.6, 0.8)),
            );

            // Use cached syntax highlighting for all lines
            let cached = state.app.devtools.cached_highlighted.borrow();
            let all_lines: Vec<&str> = code.lines().collect();
            let total = all_lines.len();

            // Pre-check which lines have associated AuraNodeIds (for hover cursor style)
            let line_map = state.app.line_to_aura_ids.borrow();
            for i in 0..total {
                let line_num = format!("{:>4}", i + 1);
                let is_highlighted = highlight_range
                    .map(|(hs, he)| i >= hs && i < he)
                    .unwrap_or(false);
                let has_aura = line_map.contains_key(&i);

                // Build line content from cached highlight spans
                let mut line_row = row![].spacing(0);
                if is_highlighted {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.8, 0.4, 0.1)),
                    );
                } else {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    );
                }

                if let Some(ref cache) = *cached {
                    if let Some(cached_line) = cache.get(i) {
                        for (fragment, color) in cached_line {
                            line_row = line_row.push(text(fragment.clone()).size(10).color(*color));
                        }
                    } else {
                        // Fallback: plain text for empty/missing cache entry
                        if let Some(line) = all_lines.get(i) {
                            line_row = line_row.push(
                                text(line.to_string())
                                    .size(10)
                                    .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                            );
                        }
                    }
                } else {
                    // No cache: plain text fallback
                    if let Some(line) = all_lines.get(i) {
                        line_row = line_row.push(
                            text(line.to_string())
                                .size(10)
                                .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                        );
                    }
                }

                // Determine background color for the line
                let bg_color = if is_highlighted {
                    iced::Color::from_rgb(1.0, 0.95, 0.85) // selected element highlight
                } else if has_aura {
                    iced::Color::from_rgb(0.94, 0.96, 1.0) // subtle blue for clickable lines
                } else {
                    iced::Color::TRANSPARENT
                };

                let line_container = container(line_row.spacing(4))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(1.0))
                    .width(iced::Length::Fill);

                // Wrap clickable lines in mouse_area for source-click → component-highlight
                if has_aura {
                    let line_idx = i;
                    let ma = mouse_area(line_container).on_press(IcedMessage {
                        widget: String::new(),
                        event: format!("{}{}", SRC_CLICK_PREFIX, line_idx),
                        input_value: None,
                    });
                    col = col.push(ma);
                } else {
                    col = col.push(line_container);
                }
            }
            drop(line_map);

            // Add edit button when element has a span and is selected
            if info.is_some() && highlight_range.is_some() {
                let edit_id = selected_id.clone().unwrap_or_default();
                col = col.push(
                    container(
                        mouse_area(
                            text("[编辑]")
                                .size(9)
                                .color(iced::Color::from_rgb(0.2, 0.5, 0.8)),
                        )
                        .on_press(IcedMessage {
                            widget: String::new(),
                            event: format!("{}{}", DEBUG_EDIT_PREFIX, edit_id),
                            input_value: None,
                        }),
                    )
                    .padding(iced::Padding::new(2.0)),
                );
            }
        }
        None => {}
    }

    // --- Edit mode UI: inline text_editor ---
    let editing = state.app.devtools.editing_element.borrow().clone();
    if let Some(ref _edit_id) = editing {
        let edit_err = state.app.devtools.edit_error.borrow().clone();
        let textarea_key = state.app.devtools.edit_textarea_key.borrow().clone();

        col = col.push(
            container(
                text("✏ 编辑源码")
                    .size(11)
                    .color(iced::Color::from_rgb(0.8, 0.3, 0.1)),
            )
            .width(iced::Length::Fill)
            .padding(iced::Padding::new(4.0)),
        );

        // Multi-line text editor using text_editor widget
        if let Some(ref key) = textarea_key {
            let content = get_textarea_content(key, "");
            let editor = text_editor(content).size(10);
            col = col.push(
                container(editor)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            1.0, 0.98, 0.92,
                        ))),
                        border: iced::Border::default()
                            .rounded(3.0)
                            .color(iced::Color::from_rgb(0.3, 0.6, 0.9))
                            .width(1.0),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(4.0))
                    .width(iced::Length::Fill),
            );
        }

        // Save / Cancel buttons
        col = col.push(
            row![
                container(
                    mouse_area(
                        container(
                            text("保存")
                                .size(10)
                                .color(iced::Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(
                                0.2, 0.6, 0.3
                            ))),
                            border: iced::Border::default().rounded(3.0),
                            ..Default::default()
                        })
                        .padding(iced::Padding::new(4.0))
                    )
                    .on_press(IcedMessage {
                        widget: String::new(),
                        event: DEBUG_EDIT_APPLY.to_string(),
                        input_value: None,
                    })
                ),
                container(
                    mouse_area(
                        container(
                            text("取消")
                                .size(10)
                                .color(iced::Color::from_rgb(0.4, 0.4, 0.4))
                        )
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(
                                0.9, 0.9, 0.9
                            ))),
                            border: iced::Border::default().rounded(3.0),
                            ..Default::default()
                        })
                        .padding(iced::Padding::new(4.0))
                    )
                    .on_press(IcedMessage {
                        widget: String::new(),
                        event: DEBUG_EDIT_CANCEL.to_string(),
                        input_value: None,
                    })
                ),
            ]
            .spacing(8),
        );

        // Show error if any
        if let Some(err) = edit_err {
            col = col.push(
                text(format!("❌ {}", err))
                    .size(9)
                    .color(iced::Color::from_rgb(0.8, 0.1, 0.1)),
            );
        }
    }

    col.into()
}

/// Apply the current edit: read edited text from textarea, write back, trigger hot reload.
pub(super) fn apply_edit(mut state: crate::ui::session::SessionViewMut) {
    let edit_elem = state.app.devtools.editing_element.borrow().clone();
    let edit_span = state.app.devtools.edit_span.borrow().clone();
    let textarea_key = state.app.devtools.edit_textarea_key.borrow().clone();

    if let (Some(_id), Some((offset, len)), Some(key)) = (edit_elem, edit_span, textarea_key) {
        // Read edited text from textarea content
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        let new_text = map
            .get(&key)
            .map(|c| c.text().to_string())
            .unwrap_or_default();
        drop(map);

        let source = state.app.source_code.borrow().clone();
        if let Some(ref code) = source {
            if offset + len <= code.len() {
                match state.component.write_source_range(offset, len, &new_text) {
                    Ok(new_code) => {
                        // Update cached source code and line offsets
                        let mut offsets = vec![0usize];
                        for (i, ch) in new_code.char_indices() {
                            if ch == '\n' {
                                offsets.push(i + 1);
                            }
                        }
                        *state.app.source_line_offsets.borrow_mut() = offsets;
                        *state.app.source_code.borrow_mut() = Some(new_code);
                        // Invalidate caches since source file changed
                        *state.app.cached_converted_view.borrow_mut() = None;
                        *state.app.devtools.cached_debug_id_map.borrow_mut() = None;
                        // Rebuild syntax highlight cache after edit
                        if let Some(ref c) = *state.app.source_code.borrow() {
                            *state.app.devtools.cached_highlighted.borrow_mut() =
                                Some(build_highlight_cache(c));
                        }
                        // Rebuild line → AuraNodeId index after edit
                        {
                            let span_map = state.component.span_map().clone();
                            if let Some(ref src) = *state.app.source_code.borrow() {
                                *state.app.line_to_aura_ids.borrow_mut() =
                                    build_line_to_aura_ids(&span_map, src);
                            }
                        }
                        // Clear edit state on success
                        *state.app.devtools.editing_element.borrow_mut() = None;
                        *state.app.devtools.edit_textarea_key.borrow_mut() = None;
                        *state.app.devtools.edit_span.borrow_mut() = None;
                        *state.app.devtools.edit_error.borrow_mut() = None;
                    }
                    Err(e) => {
                        *state.app.devtools.edit_error.borrow_mut() = Some(e);
                    }
                }
            } else {
                *state.app.devtools.edit_error.borrow_mut() =
                    Some("源码已变更，span 失效".to_string());
            }
        }
    }
}

/// Build a mapping from line number (0-based) to the list of AuraNodeIds whose spans cover that line.
/// Used for source-click → component-highlight reverse lookup.
pub(super) fn build_line_to_aura_ids(
    span_map: &std::collections::HashMap<AuraNodeId, SpanInfo>,
    source: &str,
) -> std::collections::HashMap<usize, Vec<AuraNodeId>> {
    let mut result = std::collections::HashMap::new();
    // Pre-compute byte offset of each line start
    let mut line_offsets = Vec::new();
    line_offsets.push(0);
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            line_offsets.push(i + 1);
        }
    }
    // For each AuraNodeId with a span, find the line range it covers
    for (aura_id, info) in span_map {
        if let Some((offset, len)) = info.span {
            let end = offset + len;
            // Find start line (0-based)
            let start_line = match line_offsets.binary_search(&offset) {
                Ok(line) => line,
                Err(pos) => pos.saturating_sub(1),
            };
            // Find end line
            let end_line = match line_offsets.binary_search(&end) {
                Ok(line) => line,
                Err(pos) => pos.saturating_sub(1),
            };
            let last_line = line_offsets.len().saturating_sub(1);
            for line in start_line..=end_line.min(last_line) {
                result.entry(line).or_insert_with(Vec::new).push(*aura_id);
            }
        }
    }
    result
}

/// Render the Console tab: show captured print() output.
fn render_console_tab(
    state: crate::ui::session::SessionViewRef,
) -> iced::Element<'static, IcedMessage> {
    let output = state.app.devtools.console_output.borrow();

    if output.is_empty() {
        return column![text("暂无输出")
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),]
        .into();
    }

    let mut col = column![].spacing(1);
    for line in output.iter().rev().take(100) {
        col = col.push(
            text(line.clone())
                .size(10)
                .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
        );
    }
    col.into()
}

/// A node in the debug component tree (for DevTools Elements tab).
#[derive(Clone)]
#[allow(dead_code)] // legacy component_tree subsystem; pending removal (Task 19/20)
pub(crate) struct DebugTreeNode {
    id: String,
    kind: String,
    children: Vec<DebugTreeNode>,
}

/// Debug rendering context: tracks hovered/selected widget and generates unique IDs.
pub(super) struct DebugRenderCtx {
    pub(super) hovered_id: Option<String>,
    pub(super) selected_id: Option<String>,
    /// Counter for wrapper/synthetic nodes that have no AuraNodeId.
    pub(super) wrapper_counter: std::cell::RefCell<usize>,
    /// AuraNodeId → SpanInfo (source location data).
    pub(super) span_map: std::collections::HashMap<AuraNodeId, SpanInfo>,
    /// View path → AuraNodeId (built during AuraViewBuilder conversion).
    pub(super) debug_id_map: DebugIdMap,
    /// debug element id → AuraNodeId.
    pub(super) id_to_aura: std::cell::RefCell<std::collections::HashMap<String, AuraNodeId>>,
    /// AuraNodeId → debug element id (reverse).
    pub(super) aura_to_id: std::cell::RefCell<std::collections::HashMap<AuraNodeId, String>>,
    /// Style metadata per element: id -> (kind, props, span).
    pub(super) element_styles: std::cell::RefCell<std::collections::HashMap<String, DebugElementInfo>>,
    /// Component tree stack: tracks parent-child relationships during DFS traversal.
    pub(super) tree_stack: std::cell::RefCell<Vec<DebugTreeNode>>,
    /// The final component tree root, set after rendering completes.
    pub(super) component_tree: std::cell::RefCell<Option<DebugTreeNode>>,
    /// Whether debug visualization is active (toggled by F12).
    /// When false, bounds probe containers are still created for MCP snapshot,
    /// but mouse_area / hover highlights are skipped.
    pub(super) debug_mode: bool,
    /// Plan 314 Task 4: whether to populate the `inspector_cache` (VNodeId↔iced
    /// id map, computed_style, box_model) for this frame. `debug_mode || mcp_active`
    /// — MCP capture runs without F12. Visual overlays stay on `debug_mode`.
    pub(super) capture_data: bool,
    /// Bidirectional `VNodeId <-> iced widget id` map (Plan 307 Task 12).
    /// Populated in `wrap_debug` only when `debug_mode` is true; mirrors the
    /// View-structural path scheme used by `view_to_vtree_with_paths` (Task 4)
    /// so the VNodeIds align with the live VTree. Copied into
    /// `DynamicState::live_cache` after each render for later bounds backfill
    /// (Task 13) and inspector panels (tasks 15-16).
    pub(super) inspector_cache: std::cell::RefCell<crate::ui::debug::InspectorCache>,
}

/// Debug metadata for a single UI element.
#[derive(Clone)]
pub(crate) struct DebugElementInfo {
    pub(super) kind: String,
    pub(super) props: Vec<(String, String)>,
    /// Source span: (byte_offset, byte_length) in the .at file
    pub(super) span: Option<(usize, usize)>,
}

impl DebugRenderCtx {
    /// Check if the given ID is currently hovered.
    fn is_hovered(&self, id: &str) -> bool {
        self.hovered_id.as_deref() == Some(id)
    }

    /// Begin tracking a node in the component tree (called before children are rendered).
    fn tree_enter(&self, id: String, kind: String) {
        self.tree_stack.borrow_mut().push(DebugTreeNode {
            id,
            kind,
            children: Vec::new(),
        });
    }

    /// Finish tracking a node: pop from stack, attach to parent (called after all children rendered).
    fn tree_exit(&self) {
        let node = self.tree_stack.borrow_mut().pop();
        if let Some(node) = node {
            let mut stack = self.tree_stack.borrow_mut();
            if let Some(parent) = stack.last_mut() {
                parent.children.push(node);
            } else {
                // This is the root node
                *self.component_tree.borrow_mut() = Some(node);
            }
        }
    }

    /// Wrap any element with mouse_area for hover/click detection + store style metadata.
    fn wrap_debug(
        &self,
        view_path: &[usize],
        kind: &str,
        el: iced::Element<'static, IcedMessage>,
        props: Vec<(String, String)>,
        style: Option<&Style>,
    ) -> iced::Element<'static, IcedMessage> {
        // Try to get AuraNodeId from debug_id_map
        let aura_id = self.debug_id_map.get(view_path);

        // Allocate a frame-unique counter for hover message disambiguation.
        // Also used as the fallback id index for synthetic wrapper nodes.
        let counter_val = {
            let mut c = self.wrapper_counter.borrow_mut();
            let val = *c;
            *c += 1;
            val
        };

        let (id, span) = if let Some(aura_id) = aura_id {
            // Use AuraNodeId-based ID.
            // For ForLoop iterations, the same AuraNodeId appears at different paths.
            // We must make the iced widget ID unique per path to avoid duplicate IDs
            // which cause iced to suppress rendering of duplicates.
            let base_id = format!("aura_{}", aura_id.0);
            let span_info = self.span_map.get(&aura_id);
            let id_str = if view_path.len() > 0 {
                // Include a path hash to ensure uniqueness across ForLoop iterations.
                // Use counter_val which is guaranteed unique per call.
                format!("{}_{}", base_id, counter_val)
            } else {
                base_id
            };
            let span = span_info.and_then(|info| info.span);
            // Record bidirectional mapping
            self.id_to_aura.borrow_mut().insert(id_str.clone(), aura_id);
            self.aura_to_id.borrow_mut().insert(aura_id, id_str.clone());
            // Plan 307 Task 12: record the `VNodeId <-> iced widget id` mapping.
            // `view_path` here is View-structural — the SAME scheme
            // `view_to_vtree_with_paths` (Task 4) uses to derive VTree VNodeIds,
            // so `VNodeId::new(id_from_path(&view_path_as_u16))` matches the
            // corresponding VTree node's VNodeId. Only recorded when debug mode
            // is active (the ctx only exists then, but gate defensively).
            if self.capture_data {
                let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
                let vnode_id =
                    crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(&path_u16));
                self.inspector_cache
                    .borrow_mut()
                    .set_iced_map(vnode_id, id_str.clone());
            }
            (id_str, span)
        } else {
            // Fallback: synthetic wrapper node (no AuraNodeId at this path).
            //
            // Plan 307: this branch is reached for ForLoop body nodes. The
            // tracked builder records their `debug_id_map` entry under a
            // two-segment `[iter, body]` path, but by the time the View tree
            // reaches the renderer the loop is flattened into a `Column`, so the
            // node's View-structural `view_path` is the one-segment `[k, i]` —
            // a mismatch that makes `debug_id_map.get(view_path)` return None.
            // The node is nonetheless real and present in the VTree at exactly
            // this `view_path`, so record the `VNodeId <-> id_str` mapping here
            // (mirroring the aura branch). Without it, clicking loop-body
            // widgets yields `selected_vnode = None` and an empty inspector.
            let id_str = format!("wrap_{}", counter_val);
            if self.capture_data {
                let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
                let vnode_id =
                    crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(&path_u16));
                self.inspector_cache
                    .borrow_mut()
                    .set_iced_map(vnode_id, id_str.clone());
            }
            (id_str, None)
        };

        // Plan 309 Phase 2a + 3.3: populate `computed_style` (from the parsed
        // style props) and `box_model` (declared padding/border/margin insets
        // from `IcedStyle`) for this node's cache entry. `content` is left as
        // a zero placeholder here — `backfill_bounds` refines it from the
        // measured iced rect (border-box) post-render. `props` is cloned
        // because it is moved into `element_styles` below. The VNodeId transform
        // mirrors the set_iced_map calls above, landing on the same entry the
        // inspector selects by.
        if self.capture_data {
            let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
            let vnode_id =
                crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(&path_u16));
            let (pad, border, margin) = debug_style_insets(style);
            let mut cache_ref = self.inspector_cache.borrow_mut();
            let node = cache_ref.get_mut_or_default(vnode_id);
            node.computed_style = props.clone();
            node.box_model = Some(crate::ui::debug::BoxModel {
                content: crate::ui::debug::Rect::default(),
                padding: pad,
                border,
                margin,
            });
        }

        // Track this node in the component tree
        self.tree_enter(id.clone(), kind.to_string());

        // Always store metadata (even with empty props) for component tree lookup
        self.element_styles.borrow_mut().insert(
            id.clone(),
            DebugElementInfo {
                kind: kind.to_string(),
                props,
                span,
            },
        );

        // --- Bounds probe container ---
        // For non-container elements (button, text, divider, checkbox, etc.),
        // wrap in a zero-visual container with an aura_ ID so LayoutCollector
        // can capture their rendered bounds for MCP snapshot @rect annotations.
        // Skip col/row/container/scroll — they already set IDs inside render_dynamic_view.
        //
        // Plan 402 §13.10: only wrap when F12 debug_mode is on. Previously this
        // also fired when only an MCP client was connected (capture_debug), so
        // every button sat inside a per-frame bounds-probe Container. Although
        // iced's Container is event-transparent, the extra wrapper layer
        // disrupted widget-tree state retention (button is_pressed) under the
        // ~200ms MCP heartbeat rebuild cadence, so real mouse clicks never
        // fired on_press. MCP @rect bounds are still collected under debug_mode.
        // Plan 413 follow-up: code_editor joins input/textarea in the exclusion —
        // the wrapper breaks the custom widget's fill_raw body text (verified:
        // F12 → editor body text vanishes while gutter/caret still render).
        let el: iced::Element<'static, IcedMessage> = if self.debug_mode
            && aura_id.is_some()
            && !matches!(
                kind,
                "col" | "row" | "container" | "scroll" | "input" | "textarea" | "code_editor"
            ) {
            // Use the unique id (with counter suffix) instead of raw aura_id
            // to avoid duplicate iced widget IDs from ForLoop iterations.
            container(el)
                .id(id.clone())
                .style(|_: &iced::Theme| container::Style::default())
                .into()
        } else {
            el
        };

        // --- Overlay gate ---
        // Plan 309 续篇: decouple the SELECTED highlight (orange) from the
        // inspect picker. A selection made by clicking the element-tree pane
        // (no picker engaged) must still draw its highlight on the live canvas
        // so the user sees what they're inspecting. The HOVER overlay (blue)
        // and the interactive mouse_area stay picker-only (inspect_mode) to keep
        // the canvas quiet otherwise. Both require F12 debug_mode.
        if !self.debug_mode {
            self.tree_exit();
            return el;
        }

        // Inspect-capture: inspect picker is on AND Alt is NOT held. When on,
        // interactive widgets have been built without handlers (Task 3) so this
        // capturing mouse_area can select/hover EVERY element incl. buttons.
        // Alt temporarily lifts capture (yellow box + capturing overlay off) so
        // the user can reach the native event for one interaction.
        let capture = inspect_capture_active();
        let selected = self.selected_id.as_deref() == Some(&id);
        if !capture && !selected {
            // Picker off and nothing selected → plain element (the bounds probe
            // + metadata storage above stay un-gated for MCP snapshots /
            // InspectorCache).
            self.tree_exit();
            return el;
        }

        let hovered = capture && self.is_hovered(&id);
        let move_id = format!("{}{}:{}", DEBUG_HOVER_MOVE, counter_val, id);
        let enter_msg = IcedMessage {
            widget: String::new(),
            event: move_id.clone(),
            input_value: None,
        };
        let exit_msg = IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_HOVER_EXIT, counter_val),
            input_value: None,
        };
        let press_msg = IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_SELECT_PREFIX, id),
            input_value: None,
        };
        // mouse_area interaction only in picker mode; otherwise pass the
        // bounds-probed element through so the selected border can still wrap it.
        let ma: iced::Element<'static, IcedMessage> = if capture {
            mouse_area(el)
                .on_enter(enter_msg)
                .on_exit(exit_msg)
                .on_move(move |_point| IcedMessage {
                    widget: String::new(),
                    event: move_id.clone(),
                    input_value: None,
                })
                .on_press(press_msg)
                .into()
        } else {
            el
        };

        let result: iced::Element<'static, IcedMessage> = if selected {
            // Selected element: orange border + tooltip
            let info = self.element_styles.borrow().get(&id).cloned();
            let header_text = format!("{} #{}", kind, id);
            let mut tip_col = column![text(header_text)
                .size(10)
                .color(iced::Color::from_rgb(1.0, 0.7, 0.3))]
            .spacing(1);
            if let Some(ref elem_info) = info {
                if !elem_info.props.is_empty() {
                    let mut line = String::new();
                    for (k, v) in &elem_info.props {
                        if !line.is_empty() {
                            line.push(' ');
                        }
                        line.push_str(k);
                        line.push(':');
                        line.push_str(v);
                    }
                    tip_col = tip_col.push(
                        text(line)
                            .size(9)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    );
                }
            }
            let tip_content = container(tip_col)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.15, 0.15, 0.18, 0.95,
                    ))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.8, 0.5, 0.2),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(iced::Padding::new(6.0));

            let bordered = container(ma).style(|_: &iced::Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(1.0, 0.6, 0.2),
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

            tooltip(bordered, tip_content, tooltip::Position::Top)
                .gap(4.0)
                .into()
        } else if hovered {
            // Build tooltip content from stored metadata
            let info = self.element_styles.borrow().get(&id).cloned();
            let header_text = format!("{} #{}", kind, id);
            let mut tip_col = column![text(header_text)
                .size(10)
                .color(iced::Color::from_rgb(0.4, 0.7, 1.0))]
            .spacing(1);
            if let Some(ref elem_info) = info {
                if !elem_info.props.is_empty() {
                    let mut line = String::new();
                    for (k, v) in &elem_info.props {
                        if !line.is_empty() {
                            line.push(' ');
                        }
                        line.push_str(k);
                        line.push(':');
                        line.push_str(v);
                    }
                    tip_col = tip_col.push(
                        text(line)
                            .size(9)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    );
                }
            }
            let tip_content = container(tip_col)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.15, 0.15, 0.18, 0.95,
                    ))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.5, 0.8),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(iced::Padding::new(6.0));

            let bordered = container(ma).style(|_: &iced::Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.2, 0.5, 1.0),
                    width: 1.5,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

            tooltip(bordered, tip_content, tooltip::Position::Top)
                .gap(4.0)
                .into()
        } else {
            ma
        };

        self.tree_exit();
        result
    }
}

/// Render a `View<IcedMessage>` tree into Iced elements, with input text capture
/// and optional debug hover highlights.
///
/// When `debug_ctx` is `Some`, container elements (Column, Row, Container, Scrollable)
/// get wrapped in `MouseArea` for hover detection and a blue border overlay when hovered.
/// Extract declared per-side padding / border / margin insets from a `Style`
/// for the box-model (Plan 309 Phase 3.3).
///
/// Precedence per axis: explicit per-side (`padding_top` etc.) > axis
/// (`padding_x`/`padding_y`) > uniform (`padding`). Border is uniform
/// (`IcedStyle` has only `border_width`). Margin is declared-only — iced does
/// not measure it, so it is never refined from layout.
fn debug_style_insets(
    style: Option<&Style>,
) -> (
    crate::ui::debug::EdgeInsets,
    crate::ui::debug::EdgeInsets,
    crate::ui::debug::EdgeInsets,
) {
    use crate::ui::debug::EdgeInsets;
    let Some(style) = style else {
        return Default::default();
    };
    let is = IcedStyle::from_style(style);

    let px = is.padding_x.or(is.padding);
    let py = is.padding_y.or(is.padding);
    let padding = EdgeInsets::only(
        is.padding_top.or(py).unwrap_or(0.0),
        is.padding_right.or(px).unwrap_or(0.0),
        is.padding_bottom.or(py).unwrap_or(0.0),
        is.padding_left.or(px).unwrap_or(0.0),
    );

    let border = EdgeInsets::uniform(is.border_width.unwrap_or(0.0));

    let mx = is.margin_x.or(is.margin);
    let my = is.margin_y.or(is.margin);
    let margin = EdgeInsets::only(
        is.margin_top.or(my).unwrap_or(0.0),
        is.margin_right.or(mx).unwrap_or(0.0),
        is.margin_bottom.or(my).unwrap_or(0.0),
        is.margin_left.or(mx).unwrap_or(0.0),
    );

    (padding, border, margin)
}

/// Extract style properties from an IcedStyle for debug tooltip display.
fn debug_style_props(style: Option<&Style>) -> Vec<(String, String)> {
    let Some(s) = style else { return vec![] };
    let is = IcedStyle::from_style(s);
    let mut props = Vec::new();
    if let Some(ref w) = is.width {
        props.push((
            "w".into(),
            match w {
                IcedSize::Full | IcedSize::Screen => "fill".into(),
                IcedSize::FillPortion(n) => format!("portion-{}", n),
                IcedSize::Fixed(f) => format!("{}px", *f as u16),
                IcedSize::Shrink => "auto".into(),
                IcedSize::Percent(v) => format!("{}%", v),
            },
        ));
    }
    if let Some(ref h) = is.height {
        props.push((
            "h".into(),
            match h {
                IcedSize::Full | IcedSize::Screen => "fill".into(),
                IcedSize::FillPortion(n) => format!("portion-{}", n),
                IcedSize::Fixed(f) => format!("{}px", *f as u16),
                IcedSize::Shrink => "auto".into(),
                IcedSize::Percent(v) => format!("{}%", v),
            },
        ));
    }
    if let Some(p) = is.padding {
        props.push(("pad".into(), format!("{}", p as u16)));
    }
    if let Some(g) = is.gap {
        props.push(("gap".into(), format!("{}", g as u16)));
    }
    if let Some(c) = is.background_color {
        props.push((
            "bg".into(),
            format!(
                "#{:02x}{:02x}{:02x}",
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8
            ),
        ));
    }
    if let Some(c) = is.text_color {
        props.push((
            "fg".into(),
            format!(
                "#{:02x}{:02x}{:02x}",
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8
            ),
        ));
    }
    if let Some(ref fs) = is.font_size {
        let px = match fs {
            IcedFontSize::Xs => 12,
            IcedFontSize::Sm => 14,
            IcedFontSize::Base => 16,
            IcedFontSize::Lg => 18,
            IcedFontSize::Xl => 20,
            IcedFontSize::Xxl => 24,
            IcedFontSize::X3xl => 30,
            IcedFontSize::X4xl => 36,
            IcedFontSize::X5xl => 48,
            IcedFontSize::X6xl => 60,
            IcedFontSize::X7xl => 72,
            IcedFontSize::X8xl => 96,
            IcedFontSize::X9xl => 128,
        };
        props.push(("font".into(), format!("{}px", px)));
    }
    // PLAN-053 T14: 任意字号（text-[<n>px] → font_size_arbitrary）也进
    // vtree 转储——heading §7.3 档（25.3px 等）的检验面。
    if let Some(px) = is.font_size_arbitrary {
        props.push(("font".into(), format!("{}px", px as u16)));
    }
    if let Some(r) = is.border_radius {
        props.push(("radius".into(), format!("{}", r as u16)));
    }
    if let Some(w) = is.border_width {
        props.push(("border".into(), format!("{}", w as u16)));
    }
    if let Some(ref a) = is.align_items {
        props.push((
            "align".into(),
            match a {
                IcedAlign::Start => "start",
                IcedAlign::Center => "center",
                IcedAlign::End => "end",
            }
            .into(),
        ));
    }
    if let Some(ref j) = is.justify_content {
        props.push((
            "justify".into(),
            match j {
                IcedJustify::Start => "start",
                IcedJustify::Center => "center",
                IcedJustify::End => "end",
                IcedJustify::Between => "between",
                IcedJustify::Around => "around",
                IcedJustify::Evenly => "evenly",
            }
            .into(),
        ));
    }
    props
}

/// Plan 413 follow-up: diff the search pattern into the editor core.
/// Search-as-you-type: a changed non-empty pattern also jumps to the first
/// match (cosmic-edit behavior), uniformly for the VM and rust paths.
fn apply_search(storage_key: &str, search: &str) {
    use crate::ui::code_editor as ce;
    let changed =
        ce::code_editor_with(storage_key, |core| core.set_search(search)).unwrap_or(false);
    if changed && !search.is_empty() {
        ce::with_font_system(|fs| {
            ce::code_editor_with(storage_key, |core| core.find_next(fs));
        });
    }
}

/// Plan 413: build the code editor for the generic `IntoIcedElement` path
/// (messages pass through verbatim; the VM path uses
/// `build_code_editor_dynamic` which also fills `INPUT_TEXT`).
/// Plan 019 Phase 3: autodown 文档编辑器（VM 泛型路径；IcedMessage 直连
/// 路径另有 INPUT_TEXT 版本臂——payload 惯例同 code editor）。
pub(super) fn build_autodown_editor_generic<M: Clone + Debug + 'static>(
    key: &str,
    value: &str,
    is_final: bool,
    on_change: Option<M>,
    on_focus: Option<crate::ui::view::FocusCallback<M>>,
    on_link: Option<crate::ui::view::LinkCallback<M>>,
    placeholder: Option<String>,
) -> iced::Element<'static, M> {
    #[cfg(feature = "autodown")]
    {
        use crate::ui::autodown_editor as ade;
        let sk = ade::storage_key(key);
        // 单向数据流：外部差分回写（与 code editor 同 §5.4 口径）。
        let fg = if crate::ui::style_iced_adapter::dark_mode() {
            ade_fg_dark()
        } else {
            ade_fg_light()
        };
        let mut widget = ade::widget::DocEditor::<M>::new(&sk, fg);
        // PLAN-107: install the backend font source before first fence sync.
        ade::autodown_editor_sync(&sk, value, is_final);
        // PLAN-048 T7（W4）：空态文案 lowering（content 空且非聚焦时
        // 编辑壳浅灰渲染）。
        if let Some(ph) = placeholder {
            widget = widget.placeholder(ph);
        }
        if let Some(msg) = on_change {
            widget = widget.on_change(move || msg.clone());
        }
        // Plan 044 T2：块聚焦读出回调直挂 widget（FocusMetrics 载荷）。
        if let Some(cb) = on_focus {
            widget = widget.on_focus(move |m| cb.call(m));
        }
        // PLAN-732：wikilink 激活回调直挂 widget（WikiLinkMetrics 双 Str
        // 载荷；per-widget-instance 结构性绑定——契约 C-04）。
        if let Some(cb) = on_link {
            widget = widget.on_link(move |m| cb.call(m));
        }
        widget.into()
    }
    #[cfg(not(feature = "autodown"))]
    {
        let _ = (key, is_final, on_focus, on_link);
        AbstractView::<M>::Text {
            content: value.to_owned(),
            style: None,
            selectable: false,
        }
        .into_iced()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_code_editor_generic<M: Clone + Debug + 'static>(
    key: &str,
    value: &str,
    lang: &str,
    line_numbers: bool,
    wrap: bool,
    vi: bool,
    highlight_current_line: bool,
    readonly: bool,
    tab_width: usize,
    font_size: f32,
    search: &str,
    on_change: Option<M>,
    on_cursor: Option<M>,
    on_context_menu: Option<M>,
) -> iced::Element<'static, M> {
    use crate::ui::code_editor as ce;

    let config = ce::CodeEditorConfig {
        lang: lang.to_owned(),
        line_numbers,
        wrap,
        vi,
        highlight_current_line,
        readonly,
        tab_width: tab_width as u16,
        font_size,
    };
    let storage_key = ce::storage_key(key);
    // Single-direction data flow: only rewrite on an external diff.
    ce::code_editor_set_text(&storage_key, value);
    apply_search(&storage_key, search);

    let mut widget = ce::iced::CodeEditor::<M>::new(&storage_key, &config);
    if let Some(msg) = on_change {
        widget = widget.on_change(move || msg.clone());
    }
    if let Some(msg) = on_cursor {
        widget = widget.on_cursor(move || msg.clone());
    }
    if let Some(msg) = on_context_menu {
        widget = widget.on_context_menu(move |_| msg.clone());
    }

    // PLAN-629 T-03: 寄宿公共 scroller（M 无关契约——draw 每帧偏移同步 +
    // dispatch_app 尾部排水光标跟随，见 render_dynamic_view 同款包裹）。
    if std::env::var("AUTO_EDITOR_NO_SCROLLER").as_deref() != Ok("1") {
        widget = widget.hosted();
        // PLAN-701 供②b: editor scroller 入 scroll controller 注册表
        // （handle = widget id = `editor-scroll-{key}`，core::editor_scroll_handle
        // 单源）——读端点 `code_editor_scroll_offset_*` 走注册表快照，写端点
        // `code_editor_scroll_to` 走 intent 排空（既有 drain 消费者）。本臂
        // M 泛型无法合成 on_scroll 回声闭包（ScrollCallback<M> 需 M 具体值）
        // ——测量走 ScrollStateReader 读回通道（bind 预热 + drain 后校正 +
        // __mcp_heartbeat 2s 节拍持续读；offset 对 editor 句柄放开回写，
        // 见 __scroll_state_read 消费者），读出投影新鲜度 ≤2s。
        let scroll_id = ce::editor_scroll_handle(key);
        crate::ui::scroll::bind_controller(&scroll_id, &scroll_id);
        let scroller = iced::widget::scrollable(widget)
            .id(iced::widget::Id::from(scroll_id))
            .style(
                |_theme: &iced::Theme, _status: iced::widget::scrollable::Status| scrollbar_style(),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        return scroller.into();
    }
    widget.into()
}

/// Extract style reference from any AbstractView variant.
pub(super) fn extract_view_style<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> Option<&Style> {
    match view {
        AbstractView::Empty => None,
        // PLAN-066: Custom 自带 style（与内置变体同待遇）。
        AbstractView::Custom { style, .. } => style.as_ref(),
        // Plan 409 §10 续 5: Overlay 本身无 style(base/content 各自带)。
        AbstractView::Overlay { .. } => None,
        // PLAN-063 T-04d-2: 锚槽无自有样式，读子件。
        AbstractView::AnchorSlot { child, .. } => extract_view_style(child),
        AbstractView::ScrollAnchorMark { child, .. } => extract_view_style(child),
        // F-UAT-2: Rich 自带 style（段落级 base 尺寸/色）。
        AbstractView::Rich { style, .. } => style.as_ref(),
        // Plan 422: Popover 的 chrome 在 content 上(anchor 各自带)。
        AbstractView::Popover { .. } => None,
        // Plan 484: MouseArea 的 style(尺寸/定位类)参与 absolute/z 判定。
        AbstractView::MouseArea { style, .. } => style.as_ref(),
        // PLAN-009 P1: terminal 的 style 参与常规定位/边距判定。
        AbstractView::Terminal { style, .. } => style.as_ref(),
        AbstractView::ManagedScrollContent { .. } => None,
        // PLAN-617 T-19: video 的 style（尺寸/定位类）参与布线判定。
        AbstractView::Video { style, .. } => style.as_ref(),
        // Plan 563: Canvas 的 style(尺寸类)同 MouseArea 参与定位判定。
        AbstractView::Canvas { style, .. } => style.as_ref(),
        AbstractView::Text { style, .. } => style.as_ref(),
        AbstractView::Button { style, .. } => style.as_ref(),
        AbstractView::Checkbox { style, .. } => style.as_ref(),
        AbstractView::Slider { style, .. } => style.as_ref(),
        AbstractView::ProgressBar { style, .. } => style.as_ref(),
        AbstractView::Image { style, .. } => style.as_ref(),
        AbstractView::ImageSurface { style, .. } => style.as_ref(),
        AbstractView::WindowThumbnail { style, .. } => style.as_ref(),
        AbstractView::WorkspacePreview { style, .. } => style.as_ref(),
        AbstractView::RemoteFrame { style, .. } => style.as_ref(),
        AbstractView::Radio { style, .. } => style.as_ref(),
        AbstractView::Select { style, .. } => style.as_ref(),
        AbstractView::Tabs { style, .. } => style.as_ref(),
        AbstractView::List { style, .. } => style.as_ref(),
        AbstractView::Table { style, .. } => style.as_ref(),
        AbstractView::Accordion { style, .. } => style.as_ref(),
        AbstractView::Sidebar { style, .. } => style.as_ref(),
        AbstractView::NavigationRail { style, .. } => style.as_ref(),
        // Container variants handled separately in render_dynamic_view
        AbstractView::Column { style, .. } => style.as_ref(),
        AbstractView::Row { style, .. } => style.as_ref(),
        AbstractView::Container { style, .. } => style.as_ref(),
        AbstractView::Scrollable { style, .. } => style.as_ref(),
        AbstractView::Input { style, .. } => style.as_ref(),
        AbstractView::Textarea { style, .. } => style.as_ref(),
        AbstractView::CodeEditor { style, .. } => style.as_ref(),
        AbstractView::AutodownEditor { style, .. } => style.as_ref(),
        AbstractView::Grid { style, .. } => style.as_ref(),
    }
}

/// PLAN-530 步骤3：Column 叠层分区——仅 absolute 子脱流（Row 分支同语义）。
/// z-index-only 子保持流内：iced 列流布局子项永不重叠，z 序无视觉效果；
/// 472 时代的"前缀 overlay"路径（0..=elev_idx 整段重渲染进叠层）使非提升
/// 子被 base + overlay 双渲染，且 base 压 compact 把正文顶进 header 之下
/// ——widgets-gallery 首页 <768 整页 ×2 叠印（W10/W11 同族）根因。
/// （[`is_elevated_view`] 的 z-index 臂与 [`extract_max_z_index`] 随该路径
/// 一并退役——仅 absolute 语义由本分区直接承载。）
/// pub(crate): plan536 探针单测直测(× 无 z 落本分区)。
pub(crate) fn column_layer_partition<M: Clone + std::fmt::Debug>(
    children: &[AbstractView<M>],
) -> (Vec<usize>, Vec<usize>) {
    let mut flow = Vec::new();
    let mut floating = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let is_abs = extract_view_style(child)
            .map(|s| s.classes.iter().any(|c| matches!(c, StyleClass::Absolute)))
            .unwrap_or(false);
        if is_abs {
            floating.push(i);
        } else {
            flow.push(i);
        }
    }
    (flow, floating)
}

/// PLAN-051 P2: Stack elevated 层是否为空内容——空层不入栈。
/// absolute 空容器(如 musk composer 的 backdrop v-html 层:VM 无 v-html
/// 渲染=纯空容器)叠在 textarea 上方会挡死其点击聚焦(C1/C2 单变量实验
/// 实证:同结构有 backdrop 即不可输,无则可输)。vue 轨该层有 v-html 内容
/// 不受影响——本判定仅存在于 VM 渲染层。
/// PLAN-022 T-03 加固:声明了背景色的容器/布局件**不再判空**——bg 即
/// 可见 chrome(021 实测:分隔条 = bg-[#8899aa] 定尺寸 div + mouse-area
/// 子件,rust codegen 丢 mouse-area 后内层空 col 臂本判定跳过整层,
/// 分隔条消失)。纯透明空层(backdrop 族无 bg 声明)语义不变。
pub(super) fn is_empty_stack_layer<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> bool {
    use AbstractView as AV;
    fn has_declared_background(style: Option<&crate::ui::style::Style>) -> bool {
        style.is_some_and(|s| {
            s.classes
                .iter()
                .any(|c| matches!(c, crate::ui::style::StyleClass::BackgroundColor(_)))
        })
    }
    match view {
        AV::Empty => true,
        AV::Text { content, .. } => content.trim().is_empty(),
        AV::Container { child, style, .. } => {
            if has_declared_background(style.as_ref()) {
                return false;
            }
            is_empty_stack_layer(child)
        }
        AV::Column {
            children, style, ..
        }
        | AV::Row {
            children, style, ..
        } => {
            if has_declared_background(style.as_ref()) {
                return false;
            }
            children.iter().all(is_empty_stack_layer)
        }
        _ => false,
    }
}

/// Short tag for a View variant, used as debug hover ID prefix.
fn view_kind<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> &'static str {
    match view {
        AbstractView::Empty => "empty",
        // PLAN-066: 原生外部组件 hover 前缀（快照 kind 是注册名本体）。
        AbstractView::Custom { .. } => "custom",
        AbstractView::Overlay { .. } => "overlay",
        AbstractView::AnchorSlot { .. } => "anchor_slot",
        AbstractView::ScrollAnchorMark { .. } => "scroll_anchor",
        AbstractView::Popover { .. } => "popover",
        AbstractView::MouseArea { .. } => "mouse_area",
        AbstractView::Canvas { .. } => "canvas",
        AbstractView::Text { .. } => "text",
        AbstractView::Rich { .. } => "rich",
        AbstractView::Button { .. } => "button",
        AbstractView::Checkbox { .. } => "checkbox",
        AbstractView::Slider { .. } => "slider",
        AbstractView::ProgressBar { .. } => "progress",
        AbstractView::Image { .. } => "image",
        AbstractView::Video { .. } => "video",
        AbstractView::ImageSurface { .. } => "image_surface",
        AbstractView::WindowThumbnail { .. } => "window_thumbnail",
        AbstractView::WorkspacePreview { .. } => "workspace_preview",
        AbstractView::RemoteFrame { .. } => "remote-frame",
        AbstractView::Radio { .. } => "radio",
        AbstractView::Select { .. } => "select",
        AbstractView::Tabs { .. } => "tabs",
        AbstractView::List { .. } => "list",
        AbstractView::Table { .. } => "table",
        AbstractView::Textarea { .. } => "textarea",
        AbstractView::CodeEditor { .. } => "code_editor",
        AbstractView::Terminal { .. } => "terminal",
        AbstractView::ManagedScrollContent { .. } => "managed_content",
        AbstractView::AutodownEditor { .. } => "autodown_editor",
        AbstractView::Input { .. } => "input",
        AbstractView::Accordion { .. } => "accordion",
        AbstractView::Sidebar { .. } => "sidebar",
        AbstractView::NavigationRail { .. } => "navrail",
        AbstractView::Column { .. }
        | AbstractView::Row { .. }
        | AbstractView::Container { .. }
        | AbstractView::Scrollable { .. } => "el",
        AbstractView::Grid { .. } => "grid",
    }
}

/// Plan 483: 依 render_dynamic_view 的 DFS 序收集 Input 的派生 Id,登记进
/// `AppState::devtools::input_ids`(dynamic_view 每次脏重建清填)。聚焦
/// 路径(__focus_input / 未捕获 Tab fallback / refocus / launcher 召唤)按
/// 此寻址「当前视图的首个 input」——取代共享字面量 prompt_input 的盲聚焦。
pub(super) fn collect_input_ids(view: &AbstractView<IcedMessage>, out: &mut Vec<iced::widget::Id>) {
    match view {
        AbstractView::Input {
            placeholder,
            on_change,
            on_submit,
            width,
            password,
            ..
        } => {
            // PLAN-013 W2 修正：派生式必须与 render_dynamic_view Input 渲染
            // 臂（18859 .id(derive_input_id(input_primary...))）严格同式——
            // (widget,event) 主键优先、None 三元组兜底。此前误改 None 三元
            // 组与渲染 Id 失配，focus 永不落地（用户实测聚焦丢失）。
            let primary = on_change
                .as_ref()
                .map(|m| (m.widget.as_str(), m.event.as_str()))
                .or_else(|| {
                    on_submit
                        .as_ref()
                        .map(|m| (m.widget.as_str(), m.event.as_str()))
                });
            out.push(derive_input_id(primary, placeholder, *width, *password));
        }
        AbstractView::Column { children, .. }
        | AbstractView::Row { children, .. }
        | AbstractView::List {
            items: children, ..
        } => {
            for child in children {
                collect_input_ids(child, out);
            }
        }
        // PLAN-013 W2：MouseArea/Popover 容器穿透——launcher search input
        // 包在 scrim mouse-area 内，缺臂使收集器永远到不了它（派生 None →
        // 回退 prompt_input 死 Id → 不聚焦+无法输入的真根因）。
        AbstractView::MouseArea { content, .. } => {
            collect_input_ids(content, out);
        }
        AbstractView::Popover {
            anchor, content, ..
        } => {
            if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                collect_input_ids(w, out);
            }
            collect_input_ids(content, out);
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            collect_input_ids(child, out);
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells {
                collect_input_ids(cell, out);
            }
        }
        AbstractView::Overlay { base, content, .. } => {
            collect_input_ids(base, out);
            collect_input_ids(content, out);
        }
        AbstractView::Popover {
            anchor, content, ..
        } => {
            if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                collect_input_ids(w, out);
            }
            collect_input_ids(content, out);
        }
        _ => {}
    }
}

// =====================================================================
// PLAN-095 T-02: ImageSurface loaded/error 运行时通知
// =====================================================================

/// 当前视图中的一个 ImageSurface 通知面（converted 树上收集）。
struct MediaSurfaceRef {
    src: String,
    on_loaded: Option<IcedMessage>,
    on_error: Option<IcedMessage>,
}

impl MediaSurfaceRef {
    /// 订阅身份键：handler 身份 + src。同一实例跨重建键稳定；换 src
    /// 自然产生新键（旧键被 prune → 重挂回旧 src = 新订阅）。
    fn key(&self) -> String {
        let identity = |m: Option<&IcedMessage>| {
            m.map(|m| format!("{}\u{1}{}", m.widget, m.event))
                .unwrap_or_default()
        };
        format!(
            "{}\u{1}{}\u{1}{}",
            identity(self.on_loaded.as_ref()),
            identity(self.on_error.as_ref()),
            self.src
        )
    }
}

/// 收集当前视图中带 on_loaded/on_error 的 ImageSurface（遍历面与
/// collect_input_ids 同族——Column/Row/List/MouseArea/Popover/Container/
/// Scrollable/Grid/Overlay；更外层容器中的 surface 与 focus 登记同边界）。
fn collect_media_surfaces(view: &AbstractView<IcedMessage>, out: &mut Vec<MediaSurfaceRef>) {
    match view {
        AbstractView::ImageSurface {
            src,
            on_loaded,
            on_error,
            ..
        } => {
            out.push(MediaSurfaceRef {
                src: src.clone(),
                on_loaded: on_loaded.clone(),
                on_error: on_error.clone(),
            });
        }
        AbstractView::Column { children, .. }
        | AbstractView::Row { children, .. }
        | AbstractView::List {
            items: children, ..
        } => {
            for child in children {
                collect_media_surfaces(child, out);
            }
        }
        AbstractView::MouseArea { content, .. } => collect_media_surfaces(content, out),
        AbstractView::Popover {
            anchor, content, ..
        } => {
            if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                collect_media_surfaces(w, out);
            }
            collect_media_surfaces(content, out);
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            collect_media_surfaces(child, out);
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells {
                collect_media_surfaces(cell, out);
            }
        }
        AbstractView::Overlay { base, content, .. } => {
            collect_media_surfaces(base, out);
            collect_media_surfaces(content, out);
        }
        _ => {}
    }
}

/// PLAN-095 T-02: ImageSurface loaded/error 通知 sweep（update 尾部驱动）。
///
/// - Pending 等待不报成功；重绘/轮询/同 ticket 重复构树不重复通知
///   （单次门在 [`crate::ui::session::MediaNotifyEntry`]）。
/// - Ready 两拍语义：第一拍仅记录可消费，下一拍（其间必有渲染帧消费
///   rendition）单次派发 on_loaded——与「当前有效 ImageSurface 已在
///   渲染帧中消费可绘制 rendition」的 loaded 最低语义对齐。
/// - 终态失败（损坏/缺票/过期/后台 decode 失败）单次派发 on_error，
///   `fn(str)` 声明按实参收原因短文；永不先 loaded 再称同一请求失败。
/// - 表项随视图存在性生死：卸载/换 src 即退订，重挂载 = 新订阅。
/// - 不在 view 构建/paint 期重入 handler：派发经 on_with_input_for 在
///   update 周期同步走既有 VM 派发（与 __mcp 合成事件同通道）。
/// - 实参按 handler 声明数对齐（plan-576 D4 口径）：loaded 0..=3 形参
///   投影 []/[rev]/[w,h]/[w,h,rev]，error 0/1 形参投影 []/[原因串]；
///   超出面响亮跳过不误派。
pub(super) fn media_notify_sweep(
    component: &mut crate::ui::dynamic::DynamicComponent,
    app: &crate::ui::session::AppState,
) {
    // PLAN-811 R2（静止期注册修复）：视图收集先行——早退门只对「当前视图
    // 无带回调媒体面」生效。原先表空+pending0+代次未变即返回，会让媒体已
    // 全部静止后新挂载的 surface（031 的绑定总伴随 request_view 代次扰动，
    // 但纯静止重挂载是合法用法）永远不入表、事件永不派发。
    let (view, _, _) = component.view_with_debug_gated(false);
    let converted = convert_view_messages(view);
    let mut surfaces: Vec<MediaSurfaceRef> = Vec::new();
    collect_media_surfaces(&converted, &mut surfaces);
    let has_handled_surface = surfaces
        .iter()
        .any(|s| s.on_loaded.is_some() || s.on_error.is_some());
    let table_empty = app.media_notify.borrow().is_empty();
    let gen = auto_media_core::image_pipeline::media_change_generation();
    if !has_handled_surface
        && table_empty
        && auto_media_core::image_pipeline::pending_media_count() == 0
        && MEDIA_SWEEP_SEEN_GEN.load(std::sync::atomic::Ordering::Acquire) == gen
    {
        return;
    }
    MEDIA_SWEEP_SEEN_GEN.store(gen, std::sync::atomic::Ordering::Release);

    enum Fire {
        Loaded {
            msg: IcedMessage,
            width: u32,
            height: u32,
            revision: u64,
        },
        Error {
            msg: IcedMessage,
            reason: String,
        },
    }
    let mut fire: Vec<Fire> = Vec::new();

    {
        let mut table = app.media_notify.borrow_mut();
        let current_keys: Vec<String> = surfaces.iter().map(|s| s.key()).collect();
        table.retain(|e| current_keys.iter().any(|k| *k == e.key));
        for surf in &surfaces {
            if surf.on_loaded.is_none() && surf.on_error.is_none() {
                continue; // 无回调组件：照常显示，不入表（旧组件零改动兼容）
            }
            let key = surf.key();
            let entry = match table.iter_mut().find(|e| e.key == key) {
                Some(e) => e,
                None => {
                    table.push(crate::ui::session::MediaNotifyEntry {
                        key,
                        ready_seen: false,
                        loaded_emitted: false,
                        error_emitted: false,
                        failed_reason: None,
                    });
                    table.last_mut().expect("entry just pushed")
                }
            };
            match auto_media_core::image_pipeline::media_surface_status(&surf.src) {
                auto_media_core::image_pipeline::MediaSurfaceStatus::Ready => {
                    if !entry.ready_seen {
                        // 第一拍：只记录可消费；渲染帧在两拍之间消费 rendition。
                        entry.ready_seen = true;
                    } else if !entry.loaded_emitted {
                        entry.loaded_emitted = true;
                        if let Some(msg) = surf.on_loaded.clone() {
                            let (w, h) =
                                auto_media_core::image_pipeline::resolve_media_render(&surf.src)
                                    .map(|(_, w, h, _)| (w, h))
                                    .unwrap_or((0, 0));
                            // URI 末段 = 入队时资产 revision（parse_media_path 同形）。
                            let revision = surf
                                .src
                                .rsplit('/')
                                .next()
                                .and_then(|r| r.parse::<u64>().ok())
                                .unwrap_or(0);
                            fire.push(Fire::Loaded {
                                msg,
                                width: w,
                                height: h,
                                revision,
                            });
                        }
                    }
                }
                auto_media_core::image_pipeline::MediaSurfaceStatus::Failed(reason) => {
                    if entry.failed_reason.is_none() {
                        entry.failed_reason = Some(reason.clone());
                    }
                    if !entry.error_emitted && entry.failed_reason.is_some() {
                        entry.error_emitted = true;
                        if let Some(msg) = surf.on_error.clone() {
                            fire.push(Fire::Error { msg, reason });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for action in fire {
        match action {
            Fire::Loaded {
                msg,
                width,
                height,
                revision,
            } => {
                media_dispatch_loaded(component, &msg, width, height, revision);
            }
            Fire::Error { msg, reason } => {
                media_dispatch_error(component, &msg, &reason);
            }
        }
    }
}

/// loaded 通知派发：实参帧按声明数对齐（见 sweep 文档），payload 走
/// PAYLOAD_SEP 类型化编码，与真实输入实参同一解码口径。
fn media_dispatch_loaded(
    component: &mut crate::ui::dynamic::DynamicComponent,
    msg: &IcedMessage,
    width: u32,
    height: u32,
    revision: u64,
) {
    let (clean, _) = crate::ui::dynamic::decode_payload(&msg.event);
    let args: Vec<auto_val::Value> = match component.handler_param_count(&msg.widget, &clean) {
        Some(0) => Vec::new(),
        Some(1) => vec![auto_val::Value::Int(revision as i32)],
        Some(2) => vec![
            auto_val::Value::Int(width as i32),
            auto_val::Value::Int(height as i32),
        ],
        Some(3) => vec![
            auto_val::Value::Int(width as i32),
            auto_val::Value::Int(height as i32),
            auto_val::Value::Int(revision as i32),
        ],
        other => {
            eprintln!(
                "[VM-MEDIA] {}.{}(loaded) declares {other:?} params — media notify projects 0..=3; skipping dispatch",
                msg.widget, clean
            );
            return;
        }
    };
    let encoded = encode_payload(&clean, &args);
    component.on_with_input_for(&msg.widget, &encoded, None);
}

/// error 通知派发：`fn(str)` 声明收原因短文；0 形参不塞载荷。
fn media_dispatch_error(
    component: &mut crate::ui::dynamic::DynamicComponent,
    msg: &IcedMessage,
    reason: &str,
) {
    let (clean, _) = crate::ui::dynamic::decode_payload(&msg.event);
    let args: Vec<auto_val::Value> = match component.handler_param_count(&msg.widget, &clean) {
        Some(0) => Vec::new(),
        Some(1) => vec![auto_val::Value::str(reason)],
        other => {
            eprintln!(
                "[VM-MEDIA] {}.{}(error) declares {other:?} params — media notify projects 0..=1; skipping dispatch",
                msg.widget, clean
            );
            return;
        }
    };
    let encoded = encode_payload(&clean, &args);
    component.on_with_input_for(&msg.widget, &encoded, None);
}

/// Plan 491: 登记表焦点环遍历求址。`ids` 为 483 登记表(DFS 序 = 视觉树
/// 序);`current` 在表内 → 按方向取下/上一项(回环取模);不在表内(聚焦
/// button/textarea、条件渲染卸载)或无聚焦 → 首项(与 483「无聚焦聚焦
/// 首个」fallback 同臂)。空表返回 None(无可遍历对象,调用方不发置焦)。
pub(super) fn focus_traverse(
    ids: &[iced::widget::Id],
    current: Option<&iced::widget::Id>,
    forward: bool,
) -> Option<iced::widget::Id> {
    if ids.is_empty() {
        return None;
    }
    let step = match current.and_then(|c| ids.iter().position(|i| i == c)) {
        Some(p) if forward => p + 1,
        Some(p) => p + ids.len() - 1,
        None => 0,
    };
    Some(ids[step % ids.len()].clone())
}

/// Plan 491(T2 定案): 当前聚焦 widget Id 探针(operation)。遍历臂经
/// `operate(FindFocusedInput).then(…)` 第一段取出实际持焦者——任何来源
/// 的置焦(点击直聚/update 臂 request_focus/launcher 召唤)都落在 iced
/// widget Tree 状态里,operate 遍历可问。原 a/b 两案的取舍:a「渲染期由
/// iced 焦点状态回填 devtools」在 view() 构建期拿不到 widget Tree,本就
/// 不可行;b「五聚焦点改址时同步写」漏记点击直聚(用户点击 username 后
/// Tab 会错走无聚焦臂)。本探针即 a 的正确形态(遍历期读真实焦点态),
/// 且零新增 devtools 持久状态。与内建 `find_focused` 的关键差异:无聚焦
/// 时 finish 恒出 `Some(None)`——内建返回 `Outcome::None` 会断
/// `Task::then` 链,「无聚焦 → 聚焦首个」分支依赖恒出值。
pub(super) struct FindFocusedInput {
    focused: Option<iced::widget::Id>,
}

impl FindFocusedInput {
    pub(super) fn new() -> Self {
        Self { focused: None }
    }
}

impl iced::advanced::widget::Operation<Option<iced::widget::Id>> for FindFocusedInput {
    fn focusable(
        &mut self,
        id: Option<&iced::widget::Id>,
        _bounds: iced::Rectangle,
        state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        if state.is_focused() {
            if let Some(id) = id {
                self.focused = Some(id.clone());
            }
        }
    }

    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(
            &mut dyn iced::advanced::widget::Operation<Option<iced::widget::Id>>,
        ),
    ) {
        operate(self);
    }

    fn finish(&self) -> iced::advanced::widget::operation::Outcome<Option<iced::widget::Id>> {
        iced::advanced::widget::operation::Outcome::Some(self.focused.clone())
    }
}

pub(super) fn render_dynamic_view(
    view: AbstractView<IcedMessage>,
    debug_ctx: Option<&DebugRenderCtx>,
    path: &mut Vec<usize>,
) -> iced::Element<'static, IcedMessage> {
    // PLAN-663 C1c: 根调用点整树重写视口单位（h-screen 族在定高(px)嵌入边
    // 界内重锚定；递归子调用 path 非空，天然跳过）。重写幂等，缓存帧复用
    // 亦安全。
    let view = if path.is_empty() {
        let mut rooted = view;
        rewrite_viewport_units(&mut rooted);
        rooted
    } else {
        view
    };
    match view {
        // Overlay children can contain VM-specific columns, rows and inputs;
        // recurse through this renderer so nested content keeps the same
        // layout and message behavior as the rest of the live preview.
        AbstractView::Overlay {
            base,
            content,
            position,
        } => {
            path.push(0);
            let base_el = render_dynamic_view(*base, debug_ctx, path);
            path.pop();
            path.push(1);
            // PLAN-095 T-03: Overlay content 穿透 + 百分比几何入参（移动前取）。
            let overlay_passthrough = abs_layer_passthrough(content.as_ref());
            let overlay_units = overlay_percent_units(content.as_ref());
            let content_el = render_dynamic_view(*content, debug_ctx, path);
            path.pop();
            let floating = build_floating_layer_full(
                content_el,
                position,
                !overlay_passthrough,
                overlay_units,
            );
            let el: iced::Element<'static, IcedMessage> =
                iced::widget::stack![base_el, floating].into();
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "overlay", el, vec![], None)
            } else {
                el
            }
        }
        // Input needs IcedMessage-specific text capture — on_input constructs a new
        // IcedMessage with the typed text included, which the generic IntoIcedElement
        // trait cannot do since it's generic over M.
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
            let dbg_props = debug_style_props(style.as_ref());
            // Plan 483: Id 主键在 on_change/on_submit 被 move 进闭包前取好
            // (持所有权,避免后续 move 冲突)。
            let input_primary = on_change
                .as_ref()
                .map(|m| (m.widget.clone(), m.event.clone()))
                .or_else(|| {
                    on_submit
                        .as_ref()
                        .map(|m| (m.widget.clone(), m.event.clone()))
                });
            let mut input_widget = build_input_shape::<IcedMessage>(
                &placeholder,
                &value,
                width,
                password,
                style.as_ref(),
                disabled,
            );

            // Wire on_change → on_input (captures typed text).
            // In inspect-capture mode, omit the handler so the widget is
            // non-interactive and wrap_debug's mouse_area can capture hover/click.
            let on_change = if inspect_capture_active() {
                None
            } else {
                on_change
            };
            if let Some(msg) = on_change {
                let msg_clone = msg.clone();
                input_widget = input_widget.on_input(move |text| IcedMessage {
                    widget: msg_clone.widget.clone(),
                    event: msg_clone.event.clone(),
                    input_value: Some(text),
                });
            }

            // Wire on_submit → on_submit (fires on Enter key press)
            // Note: iced's on_submit takes a plain Message, not a closure
            let on_submit = if inspect_capture_active() {
                None
            } else {
                on_submit
            };
            if let Some(msg) = on_submit {
                input_widget = input_widget.on_submit(msg);
            }

            // Plan 483(取代 Plan 047 的全窗固定 "prompt_input"):每框唯一
            // 稳定 Id。固定共享 Id 会被 iced Focus operation 一次全置焦
            // (Tab→__focus_prompt / __focus_input / refocus 触发即双焦点+
            // 键盘双投递);Id 本身跨重建稳定,焦点状态由 iced Tree 槽位
            // diff 保持(Plan 047 的 on_submit 语义不受影响)。
            input_widget = input_widget.id(derive_input_id(
                input_primary
                    .as_ref()
                    .map(|(w, e)| (w.as_str(), e.as_str())),
                &placeholder,
                width,
                password,
            ));

            let el: iced::Element<'static, IcedMessage> = input_widget.into();
            let el = if let Some(ref s) = style {
                let iced_style = IcedStyle::from_style(s);
                wrap_with_margin(el, &iced_style)
            } else {
                el
            };
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "input", el, dbg_props, style.as_ref())
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
            highlight,
            ghost,
            keydown,
            keymap,
        } => {
            // PLAN-725 T-00：臂累积探针（勘定钻取）。
            let p725_arm_t = std::time::Instant::now();
            let p725_arm_r =
                || crate::ui::frame_segments::arm_acc("textarea", p725_arm_t.elapsed());
            let key = on_change
                .as_ref()
                .map(|m| format!("{}_{}", m.widget, m.event))
                .or_else(|| {
                    on_submit
                        .as_ref()
                        .map(|m| format!("{}_{}", m.widget, m.event))
                })
                .unwrap_or_else(|| format!("__textarea_{}", placeholder.len()));

            // Plan 057 续(富文本输入):带 highlight/ghost props 时走原生着色
            // 路径 —— 文字可见(逐段颜色/字重),ghost 嵌入内容并染灰;
            // 无新 props 时保持存量行为(含 text-transparent 叠加层兼容)。
            let rich = !highlight.is_empty() || !ghost.is_empty();
            // Plan 057 续(右箭头逐字接受 ghost):.at 里 onkeydown.right 只在
            // ghost 非空时生效 —— ghost 为空时移除拦截,保留 iced 默认光标右移
            // (行中编辑仍可用)。keydown 表随每次 view 转换重建,与 ghost
            // 状态同步增删。
            let mut keydown = keydown;
            // ghost 为空时移除「行内移动语义」键,回落 iced 默认光标行为。
            // right=forward-char;ctrl.f 同义;ctrl.e=end-of-line(ghost 显示时
            // 才拦截为接受全部)。完整 Emacs/Vim keymap 见后续架构改造。
            if ghost.is_empty() {
                for k in ["right", "ctrl.f", "ctrl.e"] {
                    keydown.remove(k);
                }
            }
            // Plan 446 批五 U3:超过内联编辑门槛的值降级只读预览 —— 编辑器
            // 路径(含 Content::with_text 缓冲构建,其 cosmic-text 整形代价
            // 即病理本体)整体绕开。机理与取舍见 TEXTAREA_INLINE_EDIT_CAP。
            let el: iced::Element<'static, IcedMessage> = if textarea_needs_preview(&value) {
                oversized_textarea_preview(&value)
            } else if rich {
                let content = get_textarea_content_rich(&key, &value, &ghost);
                let ph: &'static str = leaked_placeholder(&placeholder);
                // PLAN-051 P2: 同 composer 命中区修复——style 高度消费。
                let editor_height = textarea_editor_height(height, style.as_ref());
                let mut rich_editor = text_editor(content).placeholder(ph);
                // Plan 057 (ash-gui 输入焦点):稳定 Id(同存量路径)。
                rich_editor = rich_editor
                    .id(iced::widget::Id::from(format!("textarea_{}", key)))
                    .height(editor_height);
                // 文字可见:style 只取字号;着色全权交给 SpanHighlighter。
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&is) {
                        rich_editor = rich_editor.size(fs);
                    }
                }
                let settings = build_span_lines(&highlight, &value, &ghost);
                let rich_editor =
                    rich_editor.highlight_with::<SpanHighlighter>(settings, span_kind_to_format);
                let rich_editor =
                    with_textarea_keydown(rich_editor, keydown, &key, &keymap, on_change.clone());
                wire_textarea_actions(rich_editor, on_change, on_submit)
            } else {
                let content = get_textarea_content(&key, &value);
                // text_editor::placeholder borrows with the element's lifetime;
                // since content is &'static, we need a &'static str for placeholder too.
                let ph: &'static str = leaked_placeholder(&placeholder);
                // Plan 053 后续:默认单行高(此前 100px 把输入栏撑成 3-4 行高)。
                // 多行续行需显式 height prop(.at 的 max-h/field-sizing 是 CSS-only)。
                let editor_height = match height {
                    Some(h) => iced::Length::Fixed(h as f32),
                    None => iced::Length::Fixed(30.0),
                };
                let mut editor = text_editor(content).placeholder(ph);
                // Plan 057 (ash-gui 输入焦点): stable per-editor Id (derived from the
                // action key) so focus() can re-target this editor after view
                // rebuilds. The completion-suggestions row above the input row
                // toggling in/out (empty ⇄ non-empty) shifts the editor's position
                // in the widget tree, and iced's positional state diff drops focus
                // with it — the update loop refocuses via this Id (see update()).
                editor = editor
                    .id(iced::widget::Id::from(format!("textarea_{}", key)))
                    .height(editor_height);

                // Plan 057 (2.1): apply class-driven text styling.
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&is) {
                        editor = editor.size(fs);
                    }
                    if is.text_color.is_some() {
                        let value_color = is.text_color.unwrap();
                        // Plan 057 (ash-gui 光标可见): text-transparent 路径 ——
                        // 文字经 PlainText highlighter 染透明,style.value 让给
                        // 可见光标色(theme foreground)。
                        if value_color.a <= 0.001 {
                            editor = editor.highlight_with((), text_editor_text_transparent);
                            let caret_color = crate::ui::style_iced_adapter::resolve_semantic_rgb(
                                &crate::ui::style::Color::OnBackground,
                            )
                            .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                            .unwrap_or(iced::Color::WHITE);
                            editor = editor.style(move |_theme, _status| {
                                iced::widget::text_editor::Style {
                                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                                    border: iced::Border::default(),
                                    placeholder: iced::Color::from_rgba(0.55, 0.58, 0.65, 0.7),
                                    value: caret_color,
                                    selection: iced::Color::from_rgba(0.3, 0.5, 0.9, 0.35),
                                }
                            });
                        } else {
                            editor = editor.style(move |_theme, _status| {
                                iced::widget::text_editor::Style {
                                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                                    border: iced::Border::default(),
                                    placeholder: iced::Color::from_rgba(0.55, 0.58, 0.65, 0.7),
                                    value: value_color,
                                    selection: iced::Color::from_rgba(0.3, 0.5, 0.9, 0.35),
                                }
                            });
                        }
                    }
                }

                let editor =
                    with_textarea_keydown(editor, keydown, &key, &keymap, on_change.clone());
                wire_textarea_actions(editor, on_change, on_submit)
            };
            let el = if let Some(ref s) = style {
                let is = IcedStyle::from_style(s);
                wrap_with_margin(el, &is)
            } else {
                el
            };
            let el_out = if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "textarea", el, vec![], None)
            } else {
                el
            };
            p725_arm_r();
            el_out
        }

        // Layout containers: recursively render children through render_dynamic_view
        // so Input/Textarea get proper IcedMessage text capture.
        AbstractView::Column {
            children,
            spacing,
            padding,
            style,
            onclick,
            on_right_click,
        } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if spacing > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), spacing.to_string()));
            }
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            // PLAN-002 B：布局件 hover 标志（样式声明了 hover: 变体才构造；
            // 与样式闭包共享，wrap 时同一实例交给 HoverArea）。
            let hover = layout_hover_flag(style.as_ref());

            // PLAN-530 步骤3：仅 absolute 子脱流入叠层（Row 分支同语义）。
            // z-index-only 子保持流内——前缀 overlay 双渲染路径随 472 立场
            // 退役（见 column_layer_partition 文档）。
            // PLAN-642 T-11: distributed 列（justify-between 族）子项 grow
            // 剥离,防 iced 0.14 配给制 0×0 隐没（见 axis_fix_col_child_distributed;
            // 与 into_iced 臂同口径,Plan 319 双入口一致）。
            let distributed = style_is_distributed(style.as_ref());
            let fix_col_child = |c: AbstractView<IcedMessage>| {
                if distributed {
                    axis_fix_col_child_distributed(c)
                } else {
                    axis_fix_col_child(c)
                }
            };
            let (flow_idx, abs_idx) = column_layer_partition(&children);

            let el = if abs_idx.is_empty() {
                let mut els: Vec<iced::Element<'static, IcedMessage>> =
                    Vec::with_capacity(children.len());
                for (i, child) in children.into_iter().enumerate() {
                    path.push(i);
                    els.push(render_dynamic_view(fix_col_child(child), debug_ctx, path));
                    path.pop();
                }
                let widget_id = Some(format!(
                    "vnode_{}",
                    crate::ui::vnode::id_from_path(
                        &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                    )
                ));
                build_column(
                    els,
                    spacing,
                    padding,
                    style.as_ref(),
                    widget_id,
                    hover.clone(),
                )
            } else {
                let mut base_els: Vec<iced::Element<'static, IcedMessage>> =
                    Vec::with_capacity(flow_idx.len());
                for &i in &flow_idx {
                    path.push(i);
                    base_els.push(render_dynamic_view(
                        fix_col_child(children[i].clone()),
                        debug_ctx,
                        path,
                    ));
                    path.pop();
                }
                let widget_id = Some(format!(
                    "vnode_{}",
                    crate::ui::vnode::id_from_path(
                        &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                    )
                ));
                let base = build_column(
                    base_els,
                    spacing,
                    padding,
                    style.as_ref(),
                    widget_id,
                    hover.clone(),
                );

                let mut stk = iced::widget::Stack::new().push(base);

                for &i in &abs_idx {
                    // PLAN-051 P2: 空层不渲染不入栈(挡死下层交互件聚焦/点击)。
                    if is_empty_stack_layer(&children[i]) {
                        continue;
                    }
                    path.push(i);
                    // PLAN-095 T-03: 被动框穿透 + 百分比几何入参。
                    let passthrough_c = abs_layer_passthrough(&children[i]);
                    let units_c = overlay_percent_units(&children[i]);
                    let abs_el = render_dynamic_view(children[i].clone(), debug_ctx, path);
                    path.pop();
                    // PLAN-536 T10: 非零偏移浮层消费 offset(× 落左上根修);
                    // 零偏移(inset-0 ghost 族)保持落原点。
                    let abs_el = match dynamic_abs_layer_position(&children[i]) {
                        Some(pos) => {
                            build_floating_layer_full(abs_el, pos, !passthrough_c, units_c)
                        }
                        // PLAN-023 T-02: opaque 下沉 content 级(同 builder 臂)。
                        None if !passthrough_c => iced::widget::opaque(abs_el),
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
            let el = if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "col", el, dbg_props, style.as_ref())
            } else {
                el
            };
            // Plan 490 G4：同 Row（inspect 模式自守卫不包）。
            // PLAN-002 B：右键 + hover 同包装点。
            wrap_layout_events(el, onclick, on_right_click, hover)
        }

        // Plan 422: 锚定弹层(VM 模式路径)。子路径约定(与 snapshot/vnode/
        // find_view_by_path 的 extract_children 同序):widget 锚 anchor=0、
        // content=1;坐标锚无 anchor 子,content=0。convert_menubar 的面板项
        // 路径按 [base, popover_idx, 1, item_idx] 记录。
        AbstractView::Popover {
            anchor,
            content,
            placement,
            open,
            on_dismiss,
        } => {
            use crate::ui::iced::popover::Popover as PopoverWidget;
            use crate::ui::view::PopoverAnchor;
            let (anchor_point, anchor_is_empty, content_slot, anchor_el): (
                Option<(f32, f32)>,
                bool,
                usize,
                iced::Element<'static, IcedMessage>,
            ) = match anchor {
                PopoverAnchor::Widget(w) => {
                    let empty = matches!(&*w, AbstractView::Empty);
                    path.push(0);
                    let el = render_dynamic_view(*w, debug_ctx, path);
                    path.pop();
                    (None, empty, 1, el)
                }
                PopoverAnchor::Point { x, y } => {
                    (Some((x, y)), false, 0, iced::widget::Space::new().into())
                }
            };
            path.push(content_slot);
            let content_el = render_dynamic_view(*content, debug_ctx, path);
            path.pop();
            let mut p = PopoverWidget::new(anchor_el, content_el)
                .placement(placement)
                .open(open)
                // PLAN-530 步骤8（W13）+ PLAN-534：Modal（居中）与 Edge*
                // （贴边，sheet/drawer）放置 = 模态形态（全屏遮罩 + 面板外
                // 点击整吞），与 into_iced 臂同口径。
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
            let el: iced::Element<'static, IcedMessage> = p.into();
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "popover", el, vec![], None)
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
            let mut dbg_props = debug_style_props(style.as_ref());
            if spacing > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), spacing.to_string()));
            }
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            // PLAN-002 B：布局件 hover 标志（同 Column 臂）。
            let hover = layout_hover_flag(style.as_ref());
            // Plan 057 (2.2): absolute 子元素脱流叠层 —— 镜像 Column 分支(见
            // 上方 Plan 409 §10 续 6 注释)。ash-gui 的 ghost/高亮 overlay 是
            // `row{relative} > row{absolute inset-0}` 结构,此前 Row 分支无此
            // 处理,overlay 进 normal 流会把 textarea 挤开(ghost 断点)。
            // normal 子元素走 build_row 作 base;absolute 子元素叠在 base 之上
            // (stack 尺寸由 base 决定,overlay 落原点,接近 CSS absolute 语义)。
            let mut normal: Vec<(usize, AbstractView<IcedMessage>)> = Vec::new();
            let mut absolute: Vec<(
                usize,
                Option<crate::ui::view::OverlayPosition>,
                AbstractView<IcedMessage>,
            )> = Vec::new();
            for (i, child) in children.into_iter().enumerate() {
                let is_abs = extract_view_style(&child)
                    .map(|s| s.classes.iter().any(|c| matches!(c, StyleClass::Absolute)))
                    .unwrap_or(false);
                if is_abs {
                    // PLAN-536 T10: 偏移判定在 move 前取好(child 随分区 move)。
                    let pos = dynamic_abs_layer_position(&child);
                    absolute.push((i, pos, child));
                } else {
                    normal.push((i, child));
                }
            }
            let mut els: Vec<iced::Element<'static, IcedMessage>> =
                Vec::with_capacity(normal.len());
            for (i, child) in normal.into_iter() {
                path.push(i);
                els.push(render_dynamic_view(
                    axis_fix_row_child(child),
                    debug_ctx,
                    path,
                ));
                path.pop();
            }
            let widget_id = Some(format!(
                "vnode_{}",
                crate::ui::vnode::id_from_path(
                    &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                )
            ));
            let base = build_row(
                els,
                spacing,
                padding,
                style.as_ref(),
                widget_id,
                hover.clone(),
            );
            let el = if absolute.is_empty() {
                base
            } else {
                let mut stk = iced::widget::Stack::new().push(base);
                for (i, pos, child) in absolute.into_iter() {
                    // PLAN-051 P2: 空层不渲染不入栈(同 Column 站点)。
                    if is_empty_stack_layer(&child) {
                        continue;
                    }
                    path.push(i);
                    // PLAN-095 T-03: 被动框穿透 + 百分比几何入参。
                    let passthrough_d = abs_layer_passthrough(&child);
                    let units_d = overlay_percent_units(&child);
                    let abs_el = render_dynamic_view(child, debug_ctx, path);
                    path.pop();
                    // PLAN-536 T10: 同 Column 站点——非零偏移消费 offset,
                    // 零偏移(inset-0 ghost 族)保持落原点。
                    let abs_el = match pos {
                        Some(pos) => {
                            build_floating_layer_full(abs_el, pos, !passthrough_d, units_d)
                        }
                        None if !passthrough_d => iced::widget::opaque(abs_el),
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
            let el = if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "row", el, dbg_props, style.as_ref())
            } else {
                el
            };
            // Plan 490 G4：布局件点击（inspect 模式 wrap_layout_events 自守卫不包）。
            // PLAN-002 B：右键 + hover 同包装点。
            wrap_layout_events(el, onclick, on_right_click, hover)
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
            let mut dbg_props = debug_style_props(style.as_ref());
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            // PLAN-002 B：布局件 hover 标志（同 Column 臂）。
            let hover = layout_hover_flag(style.as_ref());
            if let Some(w) = width {
                dbg_props.push(("w".into(), format!("{}px", w)));
            }
            if let Some(h) = height {
                dbg_props.push(("h".into(), format!("{}px", h)));
            }
            if center_x {
                dbg_props.push(("center_x".into(), "true".into()));
            }
            if center_y {
                dbg_props.push(("center_y".into(), "true".into()));
            }
            path.push(0);
            let child_el = render_dynamic_view(*child, debug_ctx, path);
            path.pop();
            let widget_id = Some(format!(
                "vnode_{}",
                crate::ui::vnode::id_from_path(
                    &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                )
            ));
            let el = build_container(
                child_el,
                padding,
                width,
                height,
                center_x,
                center_y,
                style.as_ref(),
                widget_id,
                hover.clone(),
            );
            let el = if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "container", el, dbg_props, style.as_ref())
            } else {
                el
            };
            // Plan 490 G4：div/Container 点击（inspect 模式自守卫不包）。
            // PLAN-002 B：右键 + hover 同包装点。
            wrap_layout_events(el, onclick, on_right_click, hover)
        }

        // PLAN-656 T-06: managed content——dynamic 臂同 widget（debug 包裹
        // 保 bounds 可见性；真渲染在 widget draw）。
        AbstractView::ManagedScrollContent {
            key,
            logical_w,
            logical_h,
            ..
        } => {
            let el = crate::ui::iced::managed_content::ManagedScrollContentWidget::new(
                key.clone(),
                logical_w,
                logical_h,
            )
            .into_element();
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "managed_content", el, vec![], None)
            } else {
                el
            }
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
            let mut dbg_props = debug_style_props(style.as_ref());
            if let Some(w) = width {
                dbg_props.push(("w".into(), format!("{}px", w)));
            }
            if let Some(h) = height {
                dbg_props.push(("h".into(), format!("{}px", h)));
            }
            path.push(0);
            // PLAN-781 T-05: managed 桥标记（9922 unsupported 合同；动态臂
            // 与泛型臂同形——必须在 child move 进 render 前判定）。
            if let (Some(c), true) = (
                controller.as_ref(),
                content_is_managed_bridge(child.as_ref()),
            ) {
                crate::ui::scroll::controller::mark_controller_managed(c.key());
            }
            let child_el = render_dynamic_view(*child, debug_ctx, path);
            path.pop();
            // iced widget ID for layout bounds collection (Plan 282)
            let widget_id = Some(format!(
                "vnode_{}",
                crate::ui::vnode::id_from_path(
                    &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                )
            ));
            // Plan 057 续(自动滚动):auto_scroll 标记的主列表必须用固定 Id
            // (snap_to_end 目标)。debug 路径原本只挂 aura_N,导致
            // snap_to_end("blocklist_scroll") 找不到 widget 静默失效。
            // 代价:该 scroll 节点不再带 aura id(bounds 收集少一个节点)。
            // Plan 043 T1:offset 写入复用本路径的稳定 vnode_* id。
            let widget_id = if auto_scroll {
                Some("blocklist_scroll".to_string())
            } else {
                widget_id
            };
            // PLAN-656 T-05: controller pane 的测量缓存包装——合成单一
            // on_scroll 闭包（M 具体）：note_controller_state + 用户回调
            //（无用户回调回发 noop 消息，update 层未知事件 catch-all 忽略）。
            let on_scroll = match &controller {
                Some(c) => {
                    let key = c.key().to_string();
                    let user = on_scroll;
                    Some(crate::ui::view::ScrollCallback::new(
                        move |m: crate::ui::view::ScrollMetrics| {
                            crate::ui::scroll::note_controller_state(
                                &key,
                                (m.offset_x as f64, m.offset_y as f64),
                                (m.viewport_w as f64, m.viewport_h as f64),
                                (m.content_w as f64, m.content_h as f64),
                            );
                            match &user {
                                Some(cb) => cb.call(m),
                                None => IcedMessage {
                                    widget: String::new(),
                                    event: "__scroll_metrics_noop".to_string(),
                                    input_value: None,
                                },
                            }
                        },
                    ))
                }
                None => on_scroll,
            };
            let el = build_scrollable(
                child_el,
                width,
                height,
                style.as_ref(),
                widget_id,
                offset,
                on_scroll,
                axes,
                scrollbar_policy,
                controller.as_ref(),
            );
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "scroll", el, dbg_props, style.as_ref())
            } else {
                el
            }
        }

        // Grid: render each cell through render_dynamic_view (so nested
        // inputs get VM text capture + each cell gets wrap_debug), then hand
        // the built cells to the shared build_grid. MUST be explicit — the
        // `_ =>` catch-all below would bypass cell instrumentation.
        AbstractView::Grid {
            cols,
            gap,
            cells,
            style,
        } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if gap > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), gap.to_string()));
            }
            dbg_props.insert(0, ("cols".into(), cols.to_string()));
            let mut els: Vec<(iced::Element<'static, IcedMessage>, GridCellSpec)> =
                Vec::with_capacity(cells.len());
            for (i, cell) in cells.into_iter().enumerate() {
                path.push(i);
                let spec = grid_cell_spec(&cell);
                els.push((render_dynamic_view(cell, debug_ctx, path), spec));
                path.pop();
            }
            let widget_id = Some(format!(
                "vnode_{}",
                crate::ui::vnode::id_from_path(
                    &path.iter().map(|&s| s as u16).collect::<Vec<u16>>()
                )
            ));
            let el = build_grid(cols, gap, els, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "grid", el, dbg_props, style.as_ref())
            } else {
                el
            }
        }

        // Plan 413: code editor (VM path) — on_change 发布携带全文的新消息
        // （input_value: Some，PLAN-057 textarea 先例同款）。
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
            style,
        } => {
            // PLAN-725 T-00：臂累积探针（勘定钻取——整臂含 scrollable 包装）。
            let p725_arm_t = std::time::Instant::now();
            let p725_arm_r =
                || crate::ui::frame_segments::arm_acc("code_editor", p725_arm_t.elapsed());
            let dbg_props = debug_style_props(style.as_ref());
            use crate::ui::code_editor as ce;

            // Bridge the renderer's semantic theme into the editor theme
            // source (dark mode + accent), then build the widget.
            ce::set_theme_source(
                crate::ui::style_iced_adapter::dark_mode(),
                &crate::ui::style_iced_adapter::accent_name(),
            );

            let config = ce::CodeEditorConfig {
                lang,
                line_numbers,
                wrap,
                vi,
                highlight_current_line,
                readonly,
                tab_width: tab_width as u16,
                font_size,
            };
            let storage_key = ce::storage_key(&key);
            // PLAN-725 T-00：臂内子分段（勘定钻取——[P725-ARM] 行）。
            let p725_t_set = std::time::Instant::now();
            ce::code_editor_set_text(&storage_key, &value);
            apply_search(&storage_key, &search);
            crate::ui::frame_segments::arm_probe("ce_set_text_search", p725_t_set.elapsed());

            let p725_t_new = std::time::Instant::now();
            let mut widget = ce::iced::CodeEditor::<IcedMessage>::new(&storage_key, &config);
            crate::ui::frame_segments::arm_probe("ce_widget_new", p725_t_new.elapsed());
            if let Some(msg) = on_change.clone() {
                let sk = storage_key.clone();
                // PLAN-057: DocEditor dynamic 臂同款同修——发布携带全文的新消息
                // （input_value: Some），INPUT_TEXT 写入移除（零读者）。
                // PLAN-725 T-02：载荷静态消费判定——三消费点（$event 实参/
                // input_state_map 绑定/空 payload 首实参）均不命中时
                // input_value=None，O(doc) 全文携带退役（`.Edit(str)` 与
                // $event 契约由判定命中臂逐点保真）。
                let carry_text = editor_onchange_carries_text(&msg.event);
                widget = widget.on_change(move || {
                    if !carry_text {
                        return IcedMessage {
                            widget: msg.widget.clone(),
                            event: msg.event.clone(),
                            input_value: None,
                        };
                    }
                    // PLAN-725 T-00：S1 打点（全文 clone 形——O(doc) 载荷成本）。
                    let p725_t_s1 = std::time::Instant::now();
                    let text = ce::code_editor_text(&sk).unwrap_or_default();
                    crate::ui::frame_segments::note_s1_payload(p725_t_s1.elapsed());
                    IcedMessage {
                        widget: msg.widget.clone(),
                        event: msg.event.clone(),
                        input_value: Some(text),
                    }
                });
            }
            if let Some(msg) = on_cursor.clone() {
                widget = widget.on_cursor(move || msg.clone());
            }
            if let Some(msg) = on_context_menu.clone() {
                // Plan 422 P3: 坐标随消息携带(视口系,widget 已加 origin)——
                // encode_payload 追加 f32 参数,VM 处理器形参收 (x, y)。
                widget = widget.on_context_menu(move |pos| match pos {
                    Some((x, y)) => {
                        let (name, mut args) = crate::ui::dynamic::decode_payload(&msg.event);
                        args.push(auto_val::Value::Float(x as f64));
                        args.push(auto_val::Value::Float(y as f64));
                        IcedMessage {
                            widget: msg.widget.clone(),
                            event: crate::ui::iced::encode_payload(&name, &args),
                            input_value: None,
                        }
                    }
                    // 其他按键的 None(关菜单语义)从不被 publish;返回模板
                    // 以满足闭包类型。
                    None => msg.clone(),
                });
            }

            // PLAN-629 T-03: 寄宿公共 scroller（VM 轨主路径）——滚动条 UI/
            // 交互由官方 scrollable 承担（scrollbar_style vue 风格），编辑器
            // 只做虚拟化渲染。契约 M 无关：draw 每帧偏移同步；光标跟随经
            // core 标记由 dispatch_app 尾部排水成 scroll_to 任务。
            if std::env::var("AUTO_EDITOR_NO_SCROLLER").as_deref() != Ok("1") {
                let p725_t_tail = std::time::Instant::now();
                widget = widget.hosted();
                crate::ui::frame_segments::arm_probe("ce_hosted", p725_t_tail.elapsed());
                // PLAN-802（F802-R1-01 补齐）：VM 轨同构 bind——29480 起
                // desktop 泛型臂把 editor 寄宿 scroller 绑进 scroll
                // controller 注册表（handle = widget id，core::
                // editor_scroll_handle 单源）；本轨（render_dynamic_view）
                // 漏绑导致 scroll_to 写端点 intent 在 VM 轨永不被 drain
                // 消费（pane 缺失静默丢弃/unprimed 空转——实机探针实录）。
                // 补齐后 701 供②b 端点合同在 VM 轨闭环：读=注册表投影、
                // 写=intent 排空解析为 operation::scroll_to、读回心跳
                // ≤2s 收敛（读回臂 20652 原样）。
                let scroll_id = crate::ui::code_editor::editor_scroll_handle(&key);
                crate::ui::scroll::bind_controller(&scroll_id, &scroll_id);
                let p725_t_sc = std::time::Instant::now();
                let scroller = iced::widget::scrollable(widget)
                    .id(iced::widget::Id::from(scroll_id))
                    .style(
                        |_theme: &iced::Theme, _status: iced::widget::scrollable::Status| {
                            scrollbar_style()
                        },
                    )
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill);
                crate::ui::frame_segments::arm_probe("ce_scrollable", p725_t_sc.elapsed());
                let p725_t_wd = std::time::Instant::now();
                let el: iced::Element<'static, IcedMessage> = scroller.into();
                let el_out = if let Some(ctx) = debug_ctx {
                    ctx.wrap_debug(path, "code_editor", el, dbg_props, style.as_ref())
                } else {
                    el
                };
                crate::ui::frame_segments::arm_probe("ce_wrap_debug", p725_t_wd.elapsed());
                p725_arm_r();
                el_out
            } else {
                let el: iced::Element<'static, IcedMessage> = widget.into();
                let el_out = if let Some(ctx) = debug_ctx {
                    ctx.wrap_debug(path, "code_editor", el, dbg_props, style.as_ref())
                } else {
                    el
                };
                p725_arm_r();
                el_out
            }
        }

        // Plan 019 Phase 3: autodown doc editor (VM path) — on_change 发布携带
        // 全文的新消息（input_value: Some，PLAN-057 textarea 先例同款）。
        AbstractView::AutodownEditor {
            key,
            value,
            is_final,
            on_change,
            on_focus,
            on_link,
            placeholder,
            style: _,
        } => {
            let use_ade = cfg!(all(feature = "autodown", feature = "code-editor"));
            #[cfg(feature = "autodown")]
            {
                let dbg_props = Vec::<(&str, String)>::new();
                use crate::ui::autodown_editor as ade;
                let sk = ade::storage_key(&key);
                let fg = if crate::ui::style_iced_adapter::dark_mode() {
                    ade_fg_dark()
                } else {
                    ade_fg_light()
                };
                let mut widget = ade::widget::DocEditor::<IcedMessage>::new(&sk, fg);
                // PLAN-107: MCP may register the core before its first widget.
                ade::autodown_editor_sync(&sk, &value, is_final);
                // PLAN-048 T7（W4）：空态文案 lowering（VM 轨同 build_autodown_editor_generic）。
                if let Some(ph) = placeholder {
                    widget = widget.placeholder(ph);
                }
                if let Some(msg) = on_change.clone() {
                    let sk2 = sk.clone();
                    // PLAN-057: textarea 先例——发布携带全文的新消息（input_value:
                    // Some），解释器 on_with_input_for 才能把文本作 handler 首实参
                    // （.Edit(str)），修复真实键盘回写 state.content 断链（048）。
                    // INPUT_TEXT 写入移除：解释器路径零读者（generic 臂保持原样）。
                    // PLAN-725 T-02：载荷静态消费判定同 code_editor 臂（同族
                    // O(doc) 携带退役——契约保真口径见 input_payload_consumed）。
                    let carry_text = editor_onchange_carries_text(&msg.event);
                    widget = widget.on_change(move || {
                        if !carry_text {
                            return IcedMessage {
                                widget: msg.widget.clone(),
                                event: msg.event.clone(),
                                input_value: None,
                            };
                        }
                        let text = ade::autodown_editor_text(&sk2).unwrap_or_default();
                        IcedMessage {
                            widget: msg.widget.clone(),
                            event: msg.event.clone(),
                            input_value: Some(text),
                        }
                    });
                }
                // Plan 044 T2：VM 轨块聚焦读出——FocusMetrics 载荷直挂
                // widget（FocusCallback 已在 convert_view_messages 换型为
                // IcedMessage；onfocus 事件 → Typed 消息在 aura_view_builder
                // 装配，update 层 OnFocus 拦截直写快道消费）。
                if let Some(cb) = on_focus {
                    widget = widget.on_focus(move |m| cb.call(m));
                }
                // PLAN-732：VM 轨 wikilink 激活——LinkCallback 已在
                // convert_view_messages 换型为 IcedMessage（Typed args 双
                // Str 经 encode_payload 过 Send 边界），直挂 widget 后
                // publish → 派发 decode → DSL handler（.OpenWikiLink(str,str)）
                // 形参绑定（无 update 层拦截——真 handler 消费，契约 C-03）。
                if let Some(cb) = on_link {
                    widget = widget.on_link(move |m| cb.call(m));
                }
                let el: iced::Element<'static, IcedMessage> = widget.into();
                let _ = dbg_props;
                el
            }
            #[cfg(not(feature = "autodown"))]
            {
                // 双 feature 缺一时退化只读文本（markdown 只读轨的兜底路径）。
                let _ = (use_ade, on_focus, on_link);
                let el: iced::Element<'static, IcedMessage> = AbstractView::Text {
                    content: value,
                    style: None,
                    selectable: false,
                }
                .into_iced();
                el
            }
        }

        // PLAN-063 T-04d-2: 锚槽 → 记录布局坐标的委托 wrapper。
        AbstractView::AnchorSlot { index, child } => {
            let el = child.into_iced();
            iced::Element::new(crate::ui::anchor_slot::AnchorSlot {
                index: index as usize,
                child: el,
            })
        }
        // PLAN-781 T-03: 锚点标记 → 动态路径**递归**渲染子件（路径透传——
        // 与 vtree 透传同构，子件保留自身 vnode id/wrap_debug/采集面），
        // 外层零布局 wrapper。缺此臂时子树落泛型 into_iced（无 debug_ctx），
        // 整个子树失去 vnode id → LayoutCollector 全盲（T-03 实测定罪）。
        AbstractView::ScrollAnchorMark { pane, key, child } => {
            let el = render_dynamic_view(*child, debug_ctx, path);
            iced::Element::new(crate::ui::iced::scroll_anchor::ScrollAnchor::new(
                &pane, &key, el,
            ))
        }
        // PLAN-013 W2：MouseArea 专用臂——此前落 catch-all 泛型转换，其
        // on_input 接线不带 input_value 载荷：launcher search 等**嵌套在
        // mouse-area 内**的 input 永远拿不到文本（.SetQ(t) 实参恒空，用户
        // 实测"聚焦了但打不出字"）。IcedMessage 专用递归渲染保住
        // on_input → on_with_input_for 的文本载荷。
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
            let sized_content: iced::Element<'static, IcedMessage> = match style.as_ref() {
                Some(s) => {
                    let is = IcedStyle::from_style(s);
                    let mut c =
                        iced::widget::container(render_dynamic_view(*content, debug_ctx, path));
                    if let Some(ref ws) = is.width {
                        c = c.width(iced_length(ws));
                    }
                    if let Some(ref hs) = is.height {
                        c = c.height(iced_length(hs));
                    }
                    c.into()
                }
                None => render_dynamic_view(*content, debug_ctx, path),
            };
            let mut ma = mouse_area(sized_content);
            if let Some(msg) = on_enter {
                ma = ma.on_enter(msg);
            }
            if let Some(msg) = on_exit {
                ma = ma.on_exit(msg);
            }
            if !inspect_capture_active() {
                if let Some(msg) = on_double_click {
                    ma = ma.on_double_click(msg);
                }
                if let Some(msg) = on_click {
                    ma = ma.on_press(msg);
                }
                if let Some(msg) = on_context_menu {
                    ma = ma.on_right_press(msg);
                }
                if let Some(msg) = on_release {
                    ma = ma.on_release(msg);
                }
            }
            let inner: iced::Element<'static, IcedMessage> = ma.into();
            let wrapped: iced::Element<'static, IcedMessage> =
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
            let dbg_props = debug_style_props(style.as_ref());
            // PLAN-631 F-5：mouse-area hover 样式对（镜像 into_iced 臂与
            // 布局件臂——HoverArea + 共享标志，零 VM 消息零重建）。
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
            let el = match hover {
                Some(flag) => crate::ui::iced::hover_area::HoverArea::new(built, flag).into(),
                None => built,
            };
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, "mouse_area", el, dbg_props, style.as_ref())
            } else {
                el
            }
        }

        // Everything else delegates to the unified IntoIcedElement renderer
        _ => {
            let kind = view_kind(&view);
            // Clone the style off `view` before `into_iced()` moves it, so
            // wrap_debug can still derive box-model insets from it.
            let view_style = extract_view_style(&view).cloned();
            let dbg_props = debug_style_props(view_style.as_ref());
            let el: iced::Element<'static, IcedMessage> = view.into_iced();
            if let Some(ctx) = debug_ctx {
                ctx.wrap_debug(path, kind, el, dbg_props, view_style.as_ref())
            } else {
                el
            }
        }
    }
}

/// PLAN-739：update 主链 `input_values` 记账单源（键入消息携带全文时登记）。
/// 真实节拍回归测试（`plan739_edit_then_page_switch_view_follows_store`）
/// 驱动同一函数——update 内联记账与测试零漂移。
pub(crate) fn track_input_text(
    input_values: &mut std::collections::HashMap<String, String>,
    event_name: &str,
    text: &str,
) {
    input_values.insert(event_name.to_string(), text.to_string());
}

/// PLAN-739：update 主链 `input_values` handler 后清理单源（语义原样：
/// 保留触发事件条目 + 不在 `input_state_map` 的条目——后者含 ADE `oninput`
/// 事件，其条目对视面零作用见 [`patch_input_values`] 注记）。
pub(crate) fn retain_input_values_after_handler(
    input_values: &mut std::collections::HashMap<String, String>,
    component: &DynamicComponent,
    event_name: &str,
) {
    let input_map = component.input_state_map().clone();
    input_values.retain(|ev_name, _| ev_name == event_name || !input_map.contains_key(ev_name));
}

/// Recursively patch input View values with tracked user-typed text.
///
/// PLAN-739（F-037-R1 根修）：`AutodownEditor` **不参与本补丁**。其单一
/// 事实源是编辑器核；`value` prop 唯一消费口 `autodown_editor_sync` 已有
/// 三层守卫（`last_external` 差分快路 / `emit_document` 自回显快路 /
/// `emitted_echo` PLAN-057 回声集），覆盖本补丁的历史动机（PLAN-019 批次九
/// 防"字面 content 应用逐帧重建清焦点"——早于回声守卫成熟的遗产保护）。
/// 此前 ADE 入臂且其 `oninput` 事件不进 `input_state_map`（`scan_node_for_inputs`
/// 只扫 input/textarea/Input 标签）→ `input_values` 键入条目经 retain 永生
/// → 每帧重建把模板求值的新页内容覆写回旧页键入全文，编辑器视面被钉死
/// （jade 键入后换页粘滞冻结实证）。ADE 事件条目对视面零作用由
/// `plan739_patch_input_values_leaves_autodown_editor` 锚定。
pub(crate) fn patch_input_values(
    view: &mut AbstractView<DynamicMessage>,
    input_values: &std::collections::HashMap<String, String>,
) {
    match view {
        AbstractView::Input {
            value, on_change, ..
        }
        | AbstractView::Textarea {
            value, on_change, ..
        }
        | AbstractView::CodeEditor {
            value, on_change, ..
        } => {
            if let Some(msg) = on_change {
                let event_name = match msg {
                    DynamicMessage::Typed { event_name, .. } => event_name.clone(),
                    // PLAN-041 T-02: devtools 回显面与 Typed 同名。
                    DynamicMessage::TypedInst { event_name, .. } => event_name.clone(),
                    DynamicMessage::String(name) => name.clone(),
                };
                let clean_name = {
                    let n = event_name.trim_start_matches('.');
                    if let Some(pos) = n.rfind("::") {
                        n[pos + 2..].to_string()
                    } else {
                        n.to_string()
                    }
                };
                if let Some(text) = input_values.get(&clean_name) {
                    *value = text.clone();
                }
            }
        }
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            for child in children.iter_mut() {
                patch_input_values(child, input_values);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            patch_input_values(child, input_values);
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells.iter_mut() {
                patch_input_values(cell, input_values);
            }
        }
        AbstractView::List { items, .. } => {
            for item in items.iter_mut() {
                patch_input_values(item, input_values);
            }
        }
        AbstractView::Table { headers, rows, .. } => {
            for h in headers.iter_mut() {
                patch_input_values(h, input_values);
            }
            for row in rows.iter_mut() {
                for cell in row.iter_mut() {
                    patch_input_values(cell, input_values);
                }
            }
        }
        _ => {}
    }
}

/// Convert IcedSize to iced::Length
/// PLAN-095 T-03: 百分比尺寸 → FillPortion 单位（×10 精度，100% = 1000）。
/// 语义前提： siblings 同为 portion（浮层三段装配保证）；普通 Shrink 父
/// 语境退化为占满（记录在案）。
pub(crate) fn percent_fill_units(v: f32) -> u16 {
    (v * 10.0).round().clamp(1.0, 1000.0) as u16
}

pub(super) fn iced_length(size: &IcedSize) -> iced::Length {
    match size {
        IcedSize::Full => iced::Length::Fill,
        // PLAN-663 C1a: 视口单位缺省=满窗(Fill,历史行为)。定高(px)嵌入边界
        // 子树内已在渲染前被 rewrite_viewport_units 重写为 Fixed(边界高)。
        IcedSize::Screen => iced::Length::Fill,
        IcedSize::FillPortion(n) => iced::Length::FillPortion(*n),
        IcedSize::Fixed(px) => iced::Length::Fixed(*px),
        // PLAN-526 T27：w-auto/h-auto = hug 内容。
        IcedSize::Shrink => iced::Length::Shrink,
        // PLAN-095 T-03: 百分比 → FillPortion（浮层三段装配下=精确份额）。
        IcedSize::Percent(v) => iced::Length::FillPortion(percent_fill_units(*v)),
    }
}
/// PLAN-051 P2 追加（composer 输入框命中区）: textarea 编辑器高度解析——
/// height prop 优先;缺省时消费 style 高度(h-full → Fill,min-h → Fixed),
/// 均无才落历史缺省 30px。此前 style 高度不消费:input-compose 容器经
/// min-h 消费后有 80px 可见框,而内部 editor 恒 30px 悬在顶部——用户点
/// 击框中下部全部落在容器上,无光标无输入(musk chats composer 现场)。
pub(super) fn textarea_editor_height(height_prop: Option<u16>, style: Option<&Style>) -> iced::Length {
    if let Some(h) = height_prop {
        return iced::Length::Fixed(h as f32);
    }
    if let Some(s) = style {
        let is = IcedStyle::from_style(s);
        if let Some(ref h) = is.height {
            return iced_length(h);
        }
        if let Some(mh) = is.min_height {
            return if mh >= 9999.0 {
                iced::Length::Fill
            } else {
                iced::Length::Fixed(mh)
            };
        }
    }
    iced::Length::Fixed(30.0)
}

/// Convert IcedAlign to iced::alignment::Horizontal (for Column's align_x)
pub(super) fn iced_alignment_horizontal(align: IcedAlign) -> iced::alignment::Horizontal {
    match align {
        IcedAlign::Start => iced::alignment::Horizontal::Left,
        IcedAlign::Center => iced::alignment::Horizontal::Center,
        IcedAlign::End => iced::alignment::Horizontal::Right,
    }
}

/// Convert IcedAlign to iced::alignment::Vertical (for Row's align_y)
pub(super) fn iced_alignment_vertical(align: IcedAlign) -> iced::alignment::Vertical {
    match align {
        IcedAlign::Start => iced::alignment::Vertical::Top,
        IcedAlign::Center => iced::alignment::Vertical::Center,
        IcedAlign::End => iced::alignment::Vertical::Bottom,
    }
}

/// Extension trait for Component to add Iced-compatible view method
///
/// This allows components to be used directly with `iced::run()`.
pub trait ComponentIced: Component {
    /// Iced-compatible view function
    fn view_iced(&self) -> iced::Element<'static, Self::Msg>;

    /// Iced-compatible update function (delegates to on())
    fn update(&mut self, msg: Self::Msg) {
        self.on(msg);
    }

    /// Iced subscription, built from the backend-neutral `tick_interval_ms()`.
    /// Plan 365 W1 follow-up: moved here from `Component::subscription()` to
    /// de-ice the core trait.
    fn subscription(&self) -> iced::Subscription<Self::Msg> {
        iced::Subscription::none()
    }
}

// Blanket implementation for all Component types
impl<T: Component> ComponentIced for T
where
    T::Msg: Clone + Debug + 'static,
{
    fn view_iced(&self) -> iced::Element<'static, T::Msg> {
        // PLAN-663 C1c: rust 组件根入口同样重写视口单位（Plan 319 双入口同口径）。
        let mut view = self.view();
        rewrite_viewport_units(&mut view);
        view.into_iced()
    }
}

/// Run an auto-ui Component with Iced backend
///
/// This is the unified entry point for running UI applications with Iced.
/// Wires up the component's `subscription()` for periodic events (e.g., .Tick).
/// 002 §4 E0080 顺手修(PLAN-009 T8,R4 预授权 ≤1 人日):iced 0.14 的
/// `Subscription::map` 在 const 期检查闭包零尺寸(`check_zero_sized`),
/// 原 tick 实现 `move |_| msg.clone()` 捕获运行时消息必然编译炸(任何
/// 带 tick 的 rust-mode 应用均触发,即"两示例同炸")。改走内部包装:
/// 订阅以**枚举变体构造器**(零捕获)发 `Tick`,update 侧再经
/// `tick_msg()` 在运行时铸造真实消息。
pub fn run_app<C>() -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    run_app_with_title::<C>(None)
}

/// PLAN-010 T5: run_app with an explicit window title. iced's application
/// builder otherwise falls back to its default title ("Auto Lang - Iced"),
/// which leaks into replicated apps (at-gen). `None` keeps the historical
/// default. This is the application-chain title surface (待澄清④: 最小改,
/// 不动 VM 轨).
pub fn run_app_with_title<C>(title: Option<&str>) -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    if let Some(t) = title {
        let t: String = t.to_string();
        // PLAN-022 T-03:窗口尺寸面启动种子——thread_local 先落 startup 值,
        // 开窗后由 WindowResized 漏斗保活(无 resize 事件也有正确初值)。
        let seed = startup_window_size();
        crate::ui::style::theme::set_window_width(seed.width);
        crate::ui::style::theme::set_window_height(seed.height);
        return iced::application(
            TickWrap::<C>::default,
            TickWrap::<C>::update,
            view_wrapped::<C>,
        )
        .subscription(|c: &TickWrap<C>| {
            // Plan 407: tick 订阅(修复后形态,见上注)。
            let tick = if let Some(ms) = c.inner.tick_interval_ms() {
                iced::time::every(std::time::Duration::from_millis(ms as u64))
                    .map(|_| TickWrapMsg::<C::Msg>::Tick)
            } else {
                iced::Subscription::none()
            };
            // PLAN-022 T-03:窗口尺寸面(Resized → thread_local,见枚举注)。
            let win = iced::event::listen_with(|event, _status, _window_id| match event {
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(TickWrapMsg::<C::Msg>::WindowResized(
                        size.width,
                        size.height,
                    ))
                }
                _ => None,
            });
            iced::Subscription::batch(vec![tick, win])
        })
        .window(iced::window::Settings {
            size: seed,
            level: startup_window_level(),
            position: startup_window_position().unwrap_or_default(),
            ..Default::default()
        })
        // Plan 411 P1-C: 内嵌 Inter 三字重 + 默认 family(中文字形回退系统)。
        .font(INTER_FONT_REGULAR)
        .font(INTER_FONT_MEDIUM)
        .font(INTER_FONT_SEMIBOLD)
        .default_font(INTER_FONT)
        .title(title_fn::<C>(t))
        .run()
        .map_err(|e| e.into());
    }
    let seed = startup_window_size();
    crate::ui::style::theme::set_window_width(seed.width);
    crate::ui::style::theme::set_window_height(seed.height);
    iced::application(
        TickWrap::<C>::default,
        TickWrap::<C>::update,
        view_wrapped::<C>,
    )
    .subscription(|c: &TickWrap<C>| {
        let tick = if let Some(ms) = c.inner.tick_interval_ms() {
            iced::time::every(std::time::Duration::from_millis(ms as u64))
                .map(|_| TickWrapMsg::<C::Msg>::Tick)
        } else {
            iced::Subscription::none()
        };
        // PLAN-022 T-03:窗口尺寸面(Resized → thread_local,见枚举注)。
        let win = iced::event::listen_with(|event, _status, _window_id| match event {
            iced::Event::Window(iced::window::Event::Resized(size)) => Some(
                TickWrapMsg::<C::Msg>::WindowResized(size.width, size.height),
            ),
            _ => None,
        });
        iced::Subscription::batch(vec![tick, win])
    })
    .window(iced::window::Settings {
        size: seed,
        level: startup_window_level(),
        position: startup_window_position().unwrap_or_default(),
        ..Default::default()
    })
    .font(INTER_FONT_REGULAR)
    .font(INTER_FONT_MEDIUM)
    .font(INTER_FONT_SEMIBOLD)
    .default_font(INTER_FONT)
    .run()
    .map_err(|e| e.into())
}

/// PLAN-010 T5: 标题闭包(HRTB 由具名 fn 承载,闭包推断不过——同 view_wrapped)。
fn title_fn<C: Component>(t: String) -> impl Fn(&TickWrap<C>) -> String {
    move |_| t.clone()
}

/// 包装组件的 view 转发(HRTB 由具名 fn 承载,闭包推断不过)。
fn view_wrapped<'a, C>(component: &'a TickWrap<C>) -> iced::Element<'a, TickWrapMsg<C::Msg>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    view(&component.inner).map(TickWrapMsg::Inner)
}

/// run_app 的内部消息包装:Tick 由订阅发射,update 时转铸真实 tick 消息。
#[derive(Clone, Debug)]
enum TickWrapMsg<M: Clone + Debug> {
    Inner(M),
    Tick,
    /// PLAN-022 T-03 窗口尺寸面(逻辑 px):iced Resized 事件直通。普通
    /// run_app 路径不走 dynamic_view_impl 的尺寸同步——theme thread_local
    /// 停在默认 1024×768,mux_window_width/height(020 T-00b D7 面)读到
    /// 假值 → 分屏矩形投影 px 类几何全错(021 T-07 右面板错位蓝屏的定界
    /// 根因,evidence/022/t00-decision.md §3)。
    WindowResized(f32, f32),
}

/// run_app 的内部组件包装(DevToolsWrapper 同款手法,不动 VM 轨)。
#[derive(Debug)]
struct TickWrap<C: Component> {
    inner: C,
}

impl<C: Component + 'static> TickWrap<C>
where
    C::Msg: Clone + Debug + Send + 'static,
{
    fn update(&mut self, msg: TickWrapMsg<C::Msg>) {
        match msg {
            TickWrapMsg::Inner(m) => self.inner.on(m),
            // 运行时铸造真实 tick 消息(此处的值不进任何 const 上下文)。
            TickWrapMsg::Tick => {
                if let Some(m) = self.inner.tick_msg() {
                    self.inner.on(m);
                }
            }
            // PLAN-022 T-03:update 与 view 同在主线程 → thread_local 写读
            // 同线程,下一拍 api.mux_window_* 即真值。退化尺寸(Windows 开窗
            // /过渡态 winit 会发 Resized(0,0))拒收——0 会把投影 px 全归零
            // (分屏面板全蓝实录),保号上次真值。
            TickWrapMsg::WindowResized(w, h) => {
                if w >= 1.0 && h >= 1.0 {
                    crate::ui::style::theme::set_window_width(w);
                    crate::ui::style::theme::set_window_height(h);
                }
            }
        }
    }
}

impl<C: Component + Default> Default for TickWrap<C> {
    fn default() -> Self {
        Self {
            inner: C::default(),
        }
    }
}

impl<C: Component + 'static> Component for TickWrap<C>
where
    C::Msg: Clone + Debug + Send + 'static,
{
    type Msg = TickWrapMsg<C::Msg>;

    fn on(&mut self, msg: Self::Msg) {
        TickWrap::update(self, msg);
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        self.inner.tick_interval_ms()
    }

    /// 订阅侧用变体构造器发 `TickMsg`(见 run_app),本方法不再提供载荷。
    fn tick_msg(&self) -> Option<Self::Msg> {
        None
    }

    fn view(&self) -> crate::ui::view::View<Self::Msg> {
        self.inner.view().map_msg(TickWrapMsg::Inner)
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_val::Value> {
        self.inner.state_snapshot()
    }
}

/// Run an auto-ui Component with Iced, dispatching an initial Task after the window appears.
///
/// The boot closure is `Fn` (not `FnOnce`), so callers typically use
/// `RefCell<Option<Task>>` to consume the task on the first (and only) call.
///
/// Unlike `run_app`, this does NOT require `C: Default` — the boot closure
/// creates the state, which enables async initialization patterns.
pub fn run_app_with_task<C>(boot: impl Fn() -> (C, iced::Task<C::Msg>) + 'static) -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    iced::application(boot, C::update, view)
        .window_size(iced::Size::new(1600.0, 900.0))
        // Plan 411 P1-C: 内嵌 Inter 三字重 + 默认 family(中文字形回退系统)。
        .font(INTER_FONT_REGULAR)
        .font(INTER_FONT_MEDIUM)
        .font(INTER_FONT_SEMIBOLD)
        .default_font(INTER_FONT)
        .run()
        .map_err(|e| e.into())
}

fn view<C>(component: &C) -> iced::Element<'_, C::Msg>
where
    C: Component,
    C::Msg: Clone + Debug + 'static,
{
    component.view_iced()
}

// =====================================================================
// Plan 311: F12 DevTools for rust mode (render=rust) — MVP
// =====================================================================
//
// VM mode builds its DevTools directly inside `DynamicState` / `dynamic_view`
// / `update` (AuraNode/debug_id_map/span_map coupled). Rust mode (`run_app`)
// has none of that: `Component::subscription()` defaults to `none()`, so F12
// never reaches a handler, and there is no inspector state, VTree capture, or
// panel at all.
//
// This module adds a self-contained DevTools layer for rust mode by WRAPPING
// the user's `Component` in a `DevToolsWrapper<C>`:
//
//   - It lifts the inner `View<C::Msg>` to `View<WrapperMsg<C>>` via
//     `View::map_msg` (recursively remaps every handler), so the app keeps
//     working while the wrapper also handles DevTools messages.
//   - It builds the live VTree via the (pure, span-agnostic)
//     `view_to_vtree_with_paths(.., |_| None)` and walks the View to fill an
//     `InspectorCache` with computed style + declared box insets per node.
//   - It renders a DevTools panel (element tree + collapsible
//     盒模型/Computed/Properties sections) reusing the generic leaf helpers
//     (`kv_row`, `render_box_model_diagram`, `placeholder_panel`,
//     `layout_pending_panel`) and the pure style extractors
//     (`debug_style_insets`, `debug_style_props`).
//
// MVP scope (see docs/plans/311-rust-mode-devtools-mvp.md):
//   - Tree-selection only (click a tree node to inspect); NO canvas hover/click
//     overlay (that needs the AuraNode-coupled `wrap_debug` machinery rust mode
//     does not have).
//   - No measured layout bounds yet (bounds collection needs per-widget iced
//     ids assigned during conversion, deferred); the box model therefore shows
//     declared padding/border/margin only. Measured bounds + canvas pick are
//     documented follow-ups.

use crate::ui::vnode_converter::view_to_vtree_with_paths;

/// DevTools-only state for rust mode (Plan 311). A strict subset of
/// `DynamicState`'s DevTools fields, duplicated (not extracted) so the working
/// VM path is untouched. Interior-mutable where the iced `view` callback must
/// mutate during render.
struct DevToolsState {
    debug_mode: bool,
    devtools_open: std::cell::RefCell<bool>,
    selected_vnode: std::cell::RefCell<Option<crate::ui::vnode::VNodeId>>,
    inspector_subtab: std::cell::RefCell<InspectorSubTab>,
    inspector_sections: std::cell::RefCell<InspectorSections>,
    live_vtree: std::cell::RefCell<Option<crate::ui::vnode::VTree>>,
    live_cache: std::cell::RefCell<Option<crate::ui::debug::InspectorCache>>,
    window_size: std::cell::RefCell<iced::Size>,
    devtools_panel_width: std::cell::RefCell<f32>,
    inspector_split_ratio: std::cell::RefCell<f32>,
    dragging_inner_divider: std::cell::RefCell<bool>,
    // Plan 371 Task 11: MCP support for rust mode
    mcp_shared: std::cell::RefCell<Option<crate::ui::mcp_server::SharedStateHandle>>,
    mcp_widget_name: String,
    pub fit_pending: std::cell::Cell<bool>,
}

impl Default for DevToolsState {
    fn default() -> Self {
        // Plan 371 Task 11: start MCP server for rust mode
        let port = crate::ui::mcp_server::mcp_port();
        let widget_name = "App".to_string();
        let (mcp_shared, mcp_action_rx) =
            crate::ui::mcp_server::start_mcp_server(widget_name.clone(), port);
        mcp_shared
            .lock()
            .unwrap()
            .set_backend_kind(crate::ui::mcp_server::BackendKind::Rust);
        // Store the action receiver in the global for devtools_subscription to drain
        {
            let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
            *guard.lock().unwrap() = Some(mcp_action_rx);
        }
        DevToolsState {
            debug_mode: false,
            devtools_open: std::cell::RefCell::new(false),
            selected_vnode: std::cell::RefCell::new(None),
            inspector_subtab: std::cell::RefCell::new(InspectorSubTab::default()),
            inspector_sections: std::cell::RefCell::new(InspectorSections::default()),
            live_vtree: std::cell::RefCell::new(None),
            live_cache: std::cell::RefCell::new(None),
            window_size: std::cell::RefCell::new(startup_window_size()),
            devtools_panel_width: std::cell::RefCell::new(420.0),
            inspector_split_ratio: std::cell::RefCell::new(0.42),
            dragging_inner_divider: std::cell::RefCell::new(false),
            mcp_shared: std::cell::RefCell::new(Some(mcp_shared)),
            mcp_widget_name: widget_name,
            fit_pending: std::cell::Cell::new(startup_window_fit()),
        }
    }
}

/// Shared `DEBUG_*` string dispatch for both VM and rust DevTools (Plan 311
/// Task 2). Operates on the rust-side `DevToolsState`; VM keeps its inline copy
/// (it additionally touches VM-only fields like source spans). Returns whether
/// the UI must rebuild.
///
/// The event string may carry a `|`-delimited payload (e.g.
/// `__mouse_moved|123,456`); prefix messages (`__vnode_select_42`,
/// `__inspector_section_box`) append their argument directly with no pipe.
fn apply_debug_event(dt: &mut DevToolsState, raw: &str) -> bool {
    let (event, payload) = match raw.split_once('|') {
        Some((e, p)) => (e, p),
        None => (raw, ""),
    };

    if event == DEBUG_TOGGLE_EVENT {
        dt.debug_mode = !dt.debug_mode;
        let open = dt.debug_mode;
        *dt.devtools_open.borrow_mut() = open;
        if !open {
            *dt.selected_vnode.borrow_mut() = None;
        }
        return true;
    }

    if let Some(id_str) = event.strip_prefix(DEBUG_SELECT_VNODE_PREFIX) {
        if let Ok(value) = id_str.parse::<u64>() {
            let vnode_id = crate::ui::vnode::VNodeId::new(value);
            if *dt.selected_vnode.borrow() == Some(vnode_id) {
                *dt.selected_vnode.borrow_mut() = None;
            } else {
                *dt.selected_vnode.borrow_mut() = Some(vnode_id);
            }
        }
        return true;
    }

    if let Some(tail) = event.strip_prefix(DEBUG_INSPECTOR_SUBTAB_PREFIX) {
        if let Some(sub) = InspectorSubTab::from_message_tail(tail) {
            *dt.inspector_subtab.borrow_mut() = sub;
        }
        return true;
    }

    if let Some(tail) = event.strip_prefix(DEBUG_INSPECTOR_SECTION_PREFIX) {
        let mut s = dt.inspector_sections.borrow_mut();
        match tail {
            "box" => s.box_collapsed = !s.box_collapsed,
            "computed" => s.computed_collapsed = !s.computed_collapsed,
            "props" => s.props_collapsed = !s.props_collapsed,
            _ => {}
        }
        return true;
    }

    match event {
        "__close_devtools" => {
            *dt.devtools_open.borrow_mut() = false;
            dt.debug_mode = false;
            return true;
        }
        "__inner_divider_press" => {
            *dt.dragging_inner_divider.borrow_mut() = true;
            return true;
        }
        "__mouse_moved" => {
            if *dt.dragging_inner_divider.borrow() {
                let mut it = payload.split(',');
                let mx: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                let win_w = dt.window_size.borrow().width;
                let panel_w = (*dt.devtools_panel_width.borrow()).max(1.0);
                let panel_left = win_w - panel_w;
                let ratio = ((mx - panel_left) / panel_w).clamp(0.1, 0.9);
                *dt.inspector_split_ratio.borrow_mut() = ratio;
                return true;
            }
            return false;
        }
        "__mouse_released" => {
            if *dt.dragging_inner_divider.borrow() {
                *dt.dragging_inner_divider.borrow_mut() = false;
                return true;
            }
            return false;
        }
        "__window_resized" => {
            if let Some((w, h)) = payload.split_once('x') {
                let w: f32 = w.parse().unwrap_or(800.0);
                let h: f32 = h.parse().unwrap_or(600.0);
                *dt.window_size.borrow_mut() = iced::Size::new(w, h);
                // PLAN-022 T-03 窗口尺寸面:term.rs 的 auto.term.window_width/
                // height(mux_window_* 投影标定源)读 theme thread_local——
                // 此前仅更新 DevTools 自身 state,thread_local 恒默认
                // 1024×768 → 分屏矩形投影 px 全错(021 T-07 右面板错位
                // 蓝屏定界根因)。update 在主线程,写读同 thread_local。
                // 退化尺寸(0×0 过渡态)拒收,保号上次真值。
                if w >= 1.0 && h >= 1.0 {
                    crate::ui::style::theme::set_window_width(w);
                    crate::ui::style::theme::set_window_height(h);
                }
                // PLAN-046-B: keep .at-visible viewport height fresh.
                auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                    "vm.window_inner_height",
                    format!("{}", h),
                );
                let max_pw = w * 0.8;
                if *dt.devtools_panel_width.borrow() > max_pw {
                    *dt.devtools_panel_width.borrow_mut() = max_pw;
                }
                if dt.debug_mode {
                    return true;
                }
            }
            return false;
        }
        _ => false,
    }
}

/// Borrow the `style` field from a `View` variant (Plan 311). All widget
/// variants carry `style: Option<Style>`; this returns a reference (or `None`
/// for `Empty` / unknown variants).
fn view_style_ref<M: Clone + Debug>(view: &AbstractView<M>) -> Option<&Style> {
    match view {
        AbstractView::Text { style, .. }
        | AbstractView::Button { style, .. }
        | AbstractView::Row { style, .. }
        | AbstractView::Column { style, .. }
        | AbstractView::Input { style, .. }
        | AbstractView::Textarea { style, .. }
        | AbstractView::Checkbox { style, .. }
        | AbstractView::Container { style, .. }
        | AbstractView::Scrollable { style, .. }
        | AbstractView::Radio { style, .. }
        | AbstractView::Select { style, .. }
        | AbstractView::List { style, .. }
        | AbstractView::Table { style, .. }
        | AbstractView::Slider { style, .. }
        | AbstractView::ProgressBar { style, .. }
        | AbstractView::Accordion { style, .. }
        | AbstractView::Sidebar { style, .. }
        | AbstractView::Tabs { style, .. }
        | AbstractView::NavigationRail { style, .. }
        | AbstractView::Image { style, .. }
        | AbstractView::WindowThumbnail { style, .. }
        | AbstractView::WorkspacePreview { style, .. } => style.as_ref(),
        _ => None,
    }
}

/// Collect child views of `view` by reference, in the SAME order
/// `view_to_vtree_with_paths` descends (so the `path` → `VNodeId` derivation
/// aligns with the live VTree). Mirrors `extract_children` in
/// `vnode_converter.rs`.
pub(super) fn view_children<M: Clone + Debug>(view: &AbstractView<M>) -> Vec<&AbstractView<M>> {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            children.iter().collect()
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            vec![child.as_ref()]
        }
        AbstractView::List { items, .. } => items.iter().collect(),
        AbstractView::Table { headers, rows, .. } => {
            let mut v: Vec<&AbstractView<M>> = headers.iter().collect();
            for row in rows {
                for cell in row {
                    v.push(cell);
                }
            }
            v
        }
        AbstractView::Tabs { contents, .. } => contents.iter().collect(),
        _ => Vec::new(),
    }
}

/// Walk the live View and fill `InspectorCache` with computed style + declared
/// box insets per node (Plan 311 Task 4, simplified: no measured bounds — those
/// need per-widget iced ids, deferred). Keyed by `VNodeId = id_from_path(path)`,
/// identical to the scheme `view_to_vtree_with_paths` uses, so cache entries
/// line up with VTree nodes and the tree selection.
fn fill_cache<M: Clone + Debug>(dt: &DevToolsState, view: &AbstractView<M>) {
    let mut cache_guard = dt.live_cache.borrow_mut();
    let cache = cache_guard.get_or_insert_with(crate::ui::debug::InspectorCache::new);
    cache.clear();
    let mut path: Vec<u16> = Vec::new();
    fill_cache_rec(view, &mut path, cache);
}

fn fill_cache_rec<M: Clone + Debug>(
    view: &AbstractView<M>,
    path: &mut Vec<u16>,
    cache: &mut crate::ui::debug::InspectorCache,
) {
    let id = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path));
    let style = view_style_ref(view);
    let node = cache.get_mut_or_default(id);
    let (padding, border, margin) = debug_style_insets(style);
    if !padding.is_zero() || !border.is_zero() || !margin.is_zero() {
        node.box_model = Some(crate::ui::debug::BoxModel {
            // Measured content rect deferred (needs iced widget ids); show the
            // declared inset layers only.
            content: crate::ui::debug::Rect::new(0.0, 0.0, 0.0, 0.0),
            padding,
            border,
            margin,
        });
    }
    node.computed_style = debug_style_props(style);

    for (i, child) in view_children(view).into_iter().enumerate() {
        path.push(i as u16);
        fill_cache_rec(child, path, cache);
        path.pop();
    }
}

/// Mutable counterpart of [`view_children`]: child views by `&mut` reference, in
/// the SAME canonical order (so `VNodeId = id_from_path(path)` matches the live
/// VTree). Used by [`apply_highlight_mut`] to inject the canvas highlight without
/// touching `into_iced`.
fn view_children_mut<M: Clone + Debug>(view: &mut AbstractView<M>) -> Vec<&mut AbstractView<M>> {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            children.iter_mut().collect()
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            vec![child.as_mut()]
        }
        AbstractView::List { items, .. } => items.iter_mut().collect(),
        AbstractView::Table { headers, rows, .. } => {
            let mut v: Vec<&mut AbstractView<M>> = headers.iter_mut().collect();
            for row in rows {
                for cell in row {
                    v.push(cell);
                }
            }
            v
        }
        AbstractView::Tabs { contents, .. } => contents.iter_mut().collect(),
        _ => Vec::new(),
    }
}

/// The orange selection-outline style (Plan 311 P2-B-2). Mirrors the VM
/// `wrap_debug` selected border (`renderer.rs:5503`, `rgb(1.0,0.6,0.2)`). Built
/// via the same AURA class path the rest of the renderer uses for borders, so
/// `apply_container_style` → `build_container_style` renders it.
fn rust_highlight_style() -> Style {
    // "border-2" → BorderWidth(2.0); "border-orange-500" → BorderColor. Both
    // tokens are exercised by class-parser tests (class.rs:1302-1303).
    Style::parse("border-2 border-orange-500").unwrap_or_default()
}

/// Wrap the selected VNode's view in an orange-bordered `Container` so
/// `into_iced` draws the canvas selection outline (Plan 311 P2-B-2). Walks with
/// the SAME path scheme as `fill_cache_rec` / `view_to_vtree_with_paths`, so the
/// wrapped node is exactly the one the element-tree selection points at. The
/// `Container` adds no padding / fill, so it shrink-wraps the widget like VM's
/// raw `container(el)` border. Purely visual: the live VTree and inspector cache
/// are built from separate (un-mutated) clones, so this never perturbs them.
pub(super) fn apply_highlight_mut<M: Clone + Debug>(
    view: &mut AbstractView<M>,
    selected: Option<crate::ui::vnode::VNodeId>,
) {
    let Some(target) = selected else {
        return;
    };
    apply_highlight_mut_rec(view, &mut Vec::new(), target);
}

fn apply_highlight_mut_rec<M: Clone + Debug>(
    view: &mut AbstractView<M>,
    path: &mut Vec<u16>,
    target: crate::ui::vnode::VNodeId,
) -> bool {
    let vid = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path));
    if vid == target {
        let wrapped = AbstractView::Container {
            child: Box::new(std::mem::replace(view, AbstractView::Empty)),
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: Some(rust_highlight_style()),
            onclick: None,
            on_right_click: None,
        };
        *view = wrapped;
        return true;
    }
    for (i, child) in view_children_mut(view).into_iter().enumerate() {
        path.push(i as u16);
        let done = apply_highlight_mut_rec(child, path, target);
        path.pop();
        if done {
            return true;
        }
    }
    false
}

/// Message envelope for the rust-mode DevTools wrapper (Plan 311).
/// `Inner` carries the user component's message; `Debug` carries a
/// `DEBUG_*`-convention event string (optionally `|payload`), dispatched by
/// [`apply_debug_event`]. The `Debug` variant mirrors `IcedMessage.event` so
/// the same prefix-parsing logic is reused.
enum WrapperMsg<C: Component + 'static> {
    Inner(<C as Component>::Msg),
    Debug(String),
}

impl<C: Component + 'static> Clone for WrapperMsg<C> {
    fn clone(&self) -> Self {
        match self {
            WrapperMsg::Inner(m) => WrapperMsg::Inner(m.clone()),
            WrapperMsg::Debug(s) => WrapperMsg::Debug(s.clone()),
        }
    }
}

impl<C: Component + 'static> Debug for WrapperMsg<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrapperMsg::Inner(m) => f.debug_tuple("Inner").field(m).finish(),
            WrapperMsg::Debug(s) => f.debug_tuple("Debug").field(s).finish(),
        }
    }
}

/// Wraps a user `Component` with a rust-mode F12 DevTools layer (Plan 311).
pub struct DevToolsWrapper<C: Component + 'static> {
    inner: C,
    dt: DevToolsState,
}

impl<C: Component + Default> Default for DevToolsWrapper<C> {
    fn default() -> Self {
        DevToolsWrapper {
            inner: C::default(),
            dt: DevToolsState::default(),
        }
    }
}

/// PLAN-811 R3: merged 通知表条目——订阅身份 = handler 身份 + src（对齐
/// 动态轨 MediaSurfaceRef::key 的 canonical 规则）。
#[derive(Debug)]
struct TypedNotifyEntry {
    key: String,
    wants_loaded: bool,
    wants_error: bool,
    ready_seen: bool,
    loaded_emitted: bool,
    error_emitted: bool,
    failed_reason: Option<String>,
}

impl TypedNotifyEntry {
    /// 与动态轨 MediaNotifyEntry::awaiting_notify 同语义：还有未派发的
    /// loaded/error 时为真——媒体唤醒订阅的门控条件。
    fn awaiting_notify(&self) -> bool {
        (self.ready_seen && self.wants_loaded && !self.loaded_emitted)
            || (self.failed_reason.is_some() && self.wants_error && !self.error_emitted)
            // 尚未见到 Ready、也未见失败（资产仍 Pending）的订阅也需心跳，
            // 直到其状态落定（pending>0 全局门通常先覆盖此况）。
            || (!self.ready_seen && self.failed_reason.is_none() && (self.wants_loaded || self.wants_error))
    }
}

/// merged 轨通知表：进程级单例（run_app_devtools 每进程一个应用，与
/// 动态轨 AppState::media_notify 的每 app 表同语义作用域）。全局表让
/// 订阅回调（'static）也能读取唤醒门控。
static TYPED_NOTIFY_TABLE: std::sync::OnceLock<std::sync::Mutex<Vec<TypedNotifyEntry>>> =
    std::sync::OnceLock::new();

fn typed_notify_table() -> &'static std::sync::Mutex<Vec<TypedNotifyEntry>> {
    TYPED_NOTIFY_TABLE.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// 级联重扫标志（review R4）：sweep 派发了事件后置位——事件 handler 可能在
/// 回调内换 src/卸载/重挂载，改后的视图需要至少一次后续 sweep 完成注册/
/// 退订；该标志让唤醒门在「无 pending、无 awaiting」时也再跳一拍，直到
/// 事件链静止（无新派发即清零，空闲零消息保持）。
static TYPED_RESWEEP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// merged 侧是否有待处理媒体面（唤醒门控）：存在未终态资产、任一订阅仍在
/// 等待 loaded/error 派发、或上一拍派发过事件（视图可能已被 handler 改写）。
fn typed_media_active() -> bool {
    if auto_media_core::image_pipeline::pending_media_count() > 0 {
        return true;
    }
    if TYPED_RESWEEP.load(std::sync::atomic::Ordering::Acquire) {
        return true;
    }
    typed_notify_table()
        .lock()
        .map(|table| table.iter().any(TypedNotifyEntry::awaiting_notify))
        .unwrap_or(false)
}

/// merged 媒体独立唤醒订阅（review R3/R4）：50ms 心跳，仅在
/// `typed_media_active()` 为真时发消息；全部静止后订阅产出为零（空闲零
/// 消息）。同步与 async（with-task）启动入口共用同一装配。
fn typed_media_wake_subscription<C: Component + 'static>() -> iced::Subscription<WrapperMsg<C>>
where
    C::Msg: Send + 'static,
{
    iced::time::every(std::time::Duration::from_millis(50)).filter_map(|_| {
        typed_media_active().then(|| WrapperMsg::<C>::Debug("__media_wake__".to_string()))
    })
}

/// PLAN-811 R3: 从类型化 View 收集带回调的 ImageSurface。遍历经
/// vnode_converter::extract_children_ref（MCP 寻址同源，覆盖全部容器变体）。
fn typed_media_surfaces<M: Clone + Debug>(
    view: &AbstractView<M>,
    out: &mut Vec<(String, Option<M>, Option<M>)>,
) {
    if let AbstractView::ImageSurface {
        src,
        on_loaded,
        on_error,
        ..
    } = view
    {
        out.push((src.clone(), on_loaded.clone(), on_error.clone()));
    }
    for child in crate::ui::vnode_converter::extract_children_ref(view) {
        typed_media_surfaces(child, out);
    }
}

/// merged 通知 sweep 的代次门（镜像 MEDIA_SWEEP_SEEN_GEN）。
static TYPED_SWEEP_SEEN_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 订阅身份键：handler Debug 身份 + src（镜像动态轨 key() 形状）。
fn typed_surface_key<M: Debug>(src: &str, loaded: &Option<M>, error: &Option<M>) -> String {
    format!("{loaded:?}\u{1}{error:?}\u{1}{src}")
}

/// PLAN-811 R3: merged（静态 Component）ImageSurface loaded/error 通知。
/// 动态轨 sweep 的静态组件等价物，订阅身份/两拍/单次/随视图生死对齐
/// canonical（image-surface-events）：同 src 不同 handler = 不同订阅各
/// 单发一次；换 handler/换 src = 新订阅；同订阅重绘不重复。事件经
/// `component.on(msg)` 在本 update 周期同步派发。静止期注册：当前视图
/// 存在带回调面时早退门不生效。
fn typed_media_notify_sweep<C: Component + 'static>(component: &mut C)
where
    C::Msg: Clone + Debug + Send + 'static,
{
    let view = component.view();
    let mut surfaces: Vec<(String, Option<C::Msg>, Option<C::Msg>)> = Vec::new();
    typed_media_surfaces(&view, &mut surfaces);
    drop(view);
    let has_handled = surfaces
        .iter()
        .any(|(_, loaded, error)| loaded.is_some() || error.is_some());
    let table_empty = typed_notify_table()
        .lock()
        .map(|t| t.is_empty())
        .unwrap_or(true);
    let gen = auto_media_core::image_pipeline::media_change_generation();
    if !has_handled
        && table_empty
        && auto_media_core::image_pipeline::pending_media_count() == 0
        && TYPED_SWEEP_SEEN_GEN.load(std::sync::atomic::Ordering::Acquire) == gen
    {
        return;
    }
    TYPED_SWEEP_SEEN_GEN.store(gen, std::sync::atomic::Ordering::Release);
    // A fresh pass consumes the reschedule credit; if THIS pass fires, the
    // credit is re-earned below (handlers may rewrite the view).
    TYPED_RESWEEP.store(false, std::sync::atomic::Ordering::Release);

    let mut fire_loaded: Vec<C::Msg> = Vec::new();
    let mut fire_error: Vec<(C::Msg, String)> = Vec::new();
    let mut table = match typed_notify_table().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    let current_keys: Vec<String> = surfaces
        .iter()
        .map(|(src, loaded, error)| typed_surface_key(src, loaded, error))
        .collect();
    table.retain(|e| current_keys.iter().any(|k| *k == e.key));
    for (src, on_loaded, on_error) in &surfaces {
        if on_loaded.is_none() && on_error.is_none() {
            continue; // 无回调面：照常渲染，不入表
        }
        let key = typed_surface_key(src, on_loaded, on_error);
        let entry = match table.iter_mut().find(|e| e.key == key) {
            Some(e) => e,
            None => {
                table.push(TypedNotifyEntry {
                    key,
                    wants_loaded: on_loaded.is_some(),
                    wants_error: on_error.is_some(),
                    ready_seen: false,
                    loaded_emitted: false,
                    error_emitted: false,
                    failed_reason: None,
                });
                table.last_mut().expect("entry just pushed")
            }
        };
        match auto_media_core::image_pipeline::media_surface_status(src) {
            auto_media_core::image_pipeline::MediaSurfaceStatus::Ready => {
                if !entry.ready_seen {
                    entry.ready_seen = true; // 第一拍：下一拍（其间有渲染帧）单发
                } else if !entry.loaded_emitted {
                    entry.loaded_emitted = true;
                    if let Some(msg) = on_loaded.clone() {
                        fire_loaded.push(msg);
                    }
                }
            }
            auto_media_core::image_pipeline::MediaSurfaceStatus::Failed(reason) => {
                if entry.failed_reason.is_none() {
                    entry.failed_reason = Some(reason.clone());
                }
                if !entry.error_emitted && entry.failed_reason.is_some() {
                    entry.error_emitted = true;
                    if let Some(msg) = on_error.clone() {
                        fire_error.push((msg, reason));
                    }
                }
            }
            _ => {}
        }
    }
    drop(table);
    let fired = !fire_loaded.is_empty() || !fire_error.is_empty();
    for (msg, _) in fire_error {
        component.on(msg);
    }
    for msg in fire_loaded {
        component.on(msg);
    }
    if fired {
        // Handlers ran during this pass and may have swapped/unmounted/
        // remounted surfaces: the rewritten view needs one more sweep to
        // register/unregister — keep the wake gate alive for it.
        TYPED_RESWEEP.store(true, std::sync::atomic::Ordering::Release);
    }
}

impl<C: Component + 'static> DevToolsWrapper<C> {
    /// Construct from a pre-built inner component (Plan 311 P2-A: async-init
    /// path, where `C` is created by a boot closure rather than `Default`).
    fn from_inner(inner: C) -> Self {
        DevToolsWrapper {
            inner,
            dt: DevToolsState::default(),
        }
    }

    /// Build the full element: app (lifted to `WrapperMsg`) + DevTools panel
    /// when open. Also refreshes the live VTree + inspector cache for this
    /// frame.
    fn view_element(&self) -> iced::Element<'static, WrapperMsg<C>> {
        // Build the live VTree from the inner view (rust mode: no source spans).
        let tree =
            view_to_vtree_with_paths(self.inner.view().map_msg(WrapperMsg::<C>::Inner), |_| None);

        // Plan 371 Task 11/21: sync VTree + scalar state to MCP SharedState so
        // MCP tools (snapshot/vtree/find/exists/state) work in rust mode. The
        // state snapshot comes from `Component::state_snapshot()` (default
        // empty; the a2r generator overrides it for scalar fields).
        if let Some(ref mcp_shared) = *self.dt.mcp_shared.borrow() {
            let snap = crate::ui::mcp_server::StyledNodeSnapshot {
                widget_name: self.dt.mcp_widget_name.clone(),
                vtree: tree.clone(),
                computed: std::collections::HashMap::new(),
            };
            let mut mcp = mcp_shared.lock().unwrap();
            mcp.set_styled_vtree(snap);
            mcp.set_state(self.inner.state_snapshot());
            mcp.set_key_bindings(self.inner.key_bindings());
        }

        *self.dt.live_vtree.borrow_mut() = Some(tree);

        // Walk the view again to fill the inspector cache (style + insets).
        let mut app_view = self.inner.view().map_msg(WrapperMsg::<C>::Inner);
        fill_cache(&self.dt, &app_view);

        // P2-B-2: when the panel is open and a node is selected, wrap that node
        // in an orange-bordered Container so into_iced draws the canvas selection
        // outline. Done AFTER fill_cache (cache reflects the real tree, not the
        // highlight wrapper) and on a fresh per-frame clone (no accumulation).
        if *self.dt.devtools_open.borrow() {
            apply_highlight_mut(&mut app_view, self.dt.selected_vnode.borrow().clone());
        }

        rewrite_viewport_units(&mut app_view);
        let app_el: iced::Element<'static, WrapperMsg<C>> = app_view.into_iced();

        let app_el = if startup_window_fit() {
            container(
                scrollable(
                    container(app_el)
                        .width(iced::Length::Shrink)
                        .height(iced::Length::Shrink)
                        .id(iced::widget::Id::from("aura_fit_root_rust")),
                )
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::hidden(),
                ))
                .width(iced::Length::Shrink)
                .height(iced::Length::Shrink),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
        } else {
            app_el
        };

        if *self.dt.devtools_open.borrow() {
            let panel = rdt_devtools_panel::<C>(&self.dt);
            row![app_el, panel]
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            app_el
        }
    }
}

/// iced `view` callback for `run_app_devtools`.
fn devtools_view<C: Component + 'static>(
    w: &DevToolsWrapper<C>,
) -> iced::Element<'_, WrapperMsg<C>> {
    // 014 内存哨兵冻结态:整窗告警(消息循环已停,内存不再增长;
    // 按 F12 退出进程)。
    if crate::ui::mem_guard::is_frozen() {
        let mb = crate::ui::mem_guard::peak_mb();
        let limit = crate::ui::mem_guard::limit();
        return container(
            column![
                text("⚠ 内存超限,已暂停").size(30),
                text(format!(
                    "提交内存峰值 {mb} MB / 阈值 {limit} MB(AUTO_MEM_LIMIT_MB 可调)"
                ))
                .size(16),
                text("消息循环已冻结,现场保留;按 F12 退出进程。").size(14),
            ]
            .spacing(14),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
        .into();
    }
    w.view_element()
}

/// iced `update` callback for `run_app_devtools`.
///
/// PLAN-811 R3: the notify sweep runs at EVERY exit of the inner handler
/// (early returns included) — subscription wakes and MCP actions must not
/// skip registration/unsubscription.
fn devtools_update<C: Component + 'static>(
    w: &mut DevToolsWrapper<C>,
    msg: WrapperMsg<C>,
) -> iced::Task<WrapperMsg<C>>
where
    C::Msg: Clone + Debug + Send + 'static,
{
    let task = devtools_update_inner(w, msg);
    typed_media_notify_sweep(&mut w.inner);
    task
}

fn devtools_update_inner<C: Component + 'static>(
    w: &mut DevToolsWrapper<C>,
    msg: WrapperMsg<C>,
) -> iced::Task<WrapperMsg<C>>
where
    C::Msg: Clone + Debug + Send + 'static,
{
    // 014 内存哨兵:节流采样(1s);超限冻结后丢弃一切消息——泄漏若由
    // 消息驱动的热循环产生,立即停摆,不再吞噬内存。
    crate::ui::mem_guard::sample_and_guard();
    if crate::ui::mem_guard::is_frozen() {
        return iced::Task::none();
    }
    match msg {
        WrapperMsg::Inner(m) => w.inner.on(m),
        WrapperMsg::Debug(ref s) if s.starts_with("__fit_measured|") => {
            if let Some(payload) = s.strip_prefix("__fit_measured|") {
                let bounds: std::collections::HashMap<String, (f32, f32, f32, f32)> =
                    serde_json::from_str(payload).unwrap_or_default();
                if let Some((_x, _y, w_val, h_val)) = bounds.get("aura_fit_root_rust") {
                    if *w_val >= 50.0 && *h_val >= 50.0 {
                        w.dt.fit_pending.set(false);
                        let size = iced::Size::new((*w_val).max(200.0), (*h_val).max(200.0));
                        return iced::window::latest().then(move |opt_id| match opt_id {
                            Some(id) => iced::window::resize(id, size),
                            None => iced::Task::none(),
                        });
                    }
                }
            }
        }
        WrapperMsg::Debug(ref s) if s == "__tick__" => {
            // Plan 407: tick event — dispatch Tick to inner component.
            if let Some(msg) = w.inner.tick_msg() {
                w.inner.on(msg);
            }
            if w.dt.fit_pending.get() {
                return iced::advanced::widget::operate(crate::ui::iced::LayoutCollector::new())
                    .map(|bounds| {
                        WrapperMsg::Debug(format!(
                            "__fit_measured|{}",
                            serde_json::to_string(&bounds).unwrap_or_default()
                        ))
                    });
            }
        }
        WrapperMsg::Debug(s) => {
            // AutoUI `bind` events share the same typed dispatch path as VM.
            // Iced's keyboard::listen already excludes captured text-input
            // events, so IME composition never reaches this branch.
            if let Some(key) = s.strip_prefix("__autoui_key|") {
                if let Some(msg) = w.inner.key_message(key) {
                    w.inner.on(msg);
                }
                return iced::Task::none();
            }
            // Plan 371 Task 19: MCP action dispatch (rust mode). Two addressing
            // modes, both resolved against the inner component's typed View tree:
            //
            //   __mcp_action_path|<a,b,c>|<action>|<value>
            //       Path mode — walk the View<C::Msg> by child-index sequence to
            //       the exact node and extract its typed handler. This replaces
            //       the old Debug-substring heuristic (find_msg_by_event_name)
            //       that silently failed on labels whose first word did not
            //       substring-match an enum variant. Input text is forwarded via
            //       the thread-local INPUT_TEXT (Plan 374), which the generated
            //       `on()` reads via last_input_text().
            //
            //   __mcp_action|<widget>.<event>|<value>
            //       Event mode fallback — rarely used in rust mode (VM mode
            //       routes through a different subscription). Kept for parity.
            if let Some(rest) = s.strip_prefix("__mcp_action_path|") {
                // path|action|value  (path may be empty for the root node)
                let mut parts = rest.splitn(3, '|');
                let path_str = parts.next().unwrap_or("");
                let action_str = parts.next().unwrap_or("press");
                let value_str = parts.next().unwrap_or("");

                let path: Vec<u16> = if path_str.is_empty() {
                    Vec::new()
                } else {
                    path_str
                        .split(',')
                        .filter_map(|n| n.parse::<u16>().ok())
                        .collect()
                };
                let view = w.inner.view();
                if let Some(target) = find_view_by_path_generic(&view, &path) {
                    if !value_str.is_empty() && matches!(action_str, "type_text" | "clear") {
                        INPUT_TEXT.with(|t| *t.borrow_mut() = value_str.to_string());
                    }
                    if let Some(m) = extract_handler_from_view(target, action_str) {
                        w.inner.on(m);
                    }
                }
                return iced::Task::none();
            }
            if let Some(rest) = s.strip_prefix("__mcp_action|") {
                // Event fallback: <widget>.<event>|<value>
                let mut parts = rest.splitn(2, '|');
                let widget_event = parts.next().unwrap_or("");
                let input_value = parts.next().filter(|v| !v.is_empty());

                // MCP keyboard fallback (when the Rust component has no
                // dynamic key_bindings registry entry) arrives as
                // `key_<name>|<original key>`. Re-enter the same typed bind
                // resolver used by physical Iced keyboard events.
                if widget_event.contains(".key_") {
                    if let Some(key) = input_value {
                        if let Some(m) = w.inner.key_message(key) {
                            w.inner.on(m);
                        }
                    }
                } else {
                    // A declared `bind` is encoded by MCP as the handler name
                    // (for example `App.MoveLeft`). Resolve that name back to
                    // the physical key through the component registry, then
                    // use the same typed message hook as real Iced events.
                    let event_name = widget_event.rsplit('.').next().unwrap_or(widget_event);
                    let bindings = w.inner.key_bindings();
                    let key = bindings.iter().find_map(|(key, handler)| {
                        let handler = handler.trim_start_matches('.');
                        let handler_name = handler.rsplit('.').next().unwrap_or(handler);
                        (handler_name == event_name).then(|| key.clone())
                    });
                    if let Some(key) = key {
                        if let Some(m) = w.inner.key_message(&key) {
                            w.inner.on(m);
                        }
                    }
                }
                return iced::Task::none();
            }
            apply_debug_event(&mut w.dt, &s);
        }
    }
    iced::Task::none()
}

/// Plan 371 Task 19: walk a typed `View<M>` tree by a child-index `path`
/// (matching `VNode.path`, which is derived from `extract_children`) and return
/// a reference to the exact node. This is the precise-addressing replacement
/// for the old Debug-substring heuristic [`find_msg_by_event_name`] (removed).
pub fn find_view_by_path_generic<'a, M: Clone + Debug>(
    view: &'a AbstractView<M>,
    path: &[u16],
) -> Option<&'a AbstractView<M>> {
    let mut current = view;
    for &idx in path {
        let children = crate::ui::vnode_converter::extract_children_ref(current);
        current = children.get(idx as usize)?;
    }
    Some(current)
}

/// Plan 371 Task 19: extract the typed handler `M` from a View node, selected
/// by `action_str` (press/type_text/toggle/clear/...). Mirrors the widget->field
/// mapping the renderer uses when building iced widgets:
/// - `Button.on_click`          -> press
/// - `Input/Textarea.on_change` -> type_text / clear
/// - `Checkbox.on_toggle`       -> toggle
/// Returns `None` if the node has no handler for the requested action (e.g. a
/// layout node, or an input without an on_change binding) -- the caller then
/// silently drops the action (same observable behavior as a no-op click).
fn extract_handler_from_view<M: Clone + Debug>(
    view: &AbstractView<M>,
    action_str: &str,
) -> Option<M> {
    match (view, action_str) {
        (AbstractView::Button { onclick, .. }, "press") => Some(onclick.clone()),
        (AbstractView::Input { on_change, .. }, "type_text" | "clear") => on_change.clone(),
        // Plan 053 M4: textarea Enter/submit (mirrors Input.on_submit).
        (
            AbstractView::Input { on_submit, .. } | AbstractView::Textarea { on_submit, .. },
            "submit",
        ) => on_submit.clone(),
        (AbstractView::Textarea { on_change, .. }, "type_text" | "clear") => on_change.clone(),
        (AbstractView::CodeEditor { on_change, .. }, "type_text" | "clear") => on_change.clone(),
        (AbstractView::CodeEditor { on_cursor, .. }, "cursor") => on_cursor.clone(),
        // Plan 040 T11: type_text/clear parity with code editor (MCP typing;
        // "edit" stays the real-widget signal action).
        (AbstractView::AutodownEditor { on_change, .. }, "edit" | "type_text" | "clear") => {
            on_change.clone()
        }
        (
            AbstractView::CodeEditor {
                on_context_menu, ..
            },
            "context_menu",
        ) => on_context_menu.clone(),
        (AbstractView::Checkbox { on_toggle, .. }, "toggle") => on_toggle.clone(),
        (AbstractView::Radio { on_select, .. }, "press" | "select") => on_select.clone(),
        _ => None,
    }
}

/// Convert an Iced key press to the spelling used by AutoUI `bind` blocks.
fn rust_component_key_string(event: &iced::Event) -> Option<String> {
    let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event else {
        return None;
    };
    match key {
        iced::keyboard::Key::Named(named) => {
            let name = match named {
                iced::keyboard::key::Named::Enter => "Enter",
                iced::keyboard::key::Named::Escape => "Escape",
                iced::keyboard::key::Named::Backspace => "Backspace",
                iced::keyboard::key::Named::Tab => "Tab",
                iced::keyboard::key::Named::Space => " ",
                iced::keyboard::key::Named::ArrowUp => "ArrowUp",
                iced::keyboard::key::Named::ArrowDown => "ArrowDown",
                iced::keyboard::key::Named::ArrowLeft => "ArrowLeft",
                iced::keyboard::key::Named::ArrowRight => "ArrowRight",
                iced::keyboard::key::Named::Delete => "Delete",
                iced::keyboard::key::Named::Home => "Home",
                iced::keyboard::key::Named::End => "End",
                _ => return None,
            };
            Some(name.to_string())
        }
        iced::keyboard::Key::Character(c) => Some(c.to_string()),
        _ => None,
    }
}

/// iced `subscription` callback for `run_app_devtools`: forwards the inner
/// component's subscription (lifted to `WrapperMsg`) plus F12, window events,
/// and declarative Rust component key bindings.
fn devtools_subscription<C: Component + 'static>(
    w: &DevToolsWrapper<C>,
) -> iced::Subscription<WrapperMsg<C>>
where
    C::Msg: Send + 'static,
{
    let inner = w.inner.subscription().map(WrapperMsg::Inner);
    // Plan 371 Task 19: drain MCP actions into WrapperMsg::Debug events.
    // Two addressing modes, encoded as distinct prefixes so devtools_update
    // can dispatch without string ambiguity:
    //   Event (VM mode):   __mcp_action|<widget>.<event>|<value>
    //   Path  (rust mode): __mcp_action_path|<a,b,c>|<action>|<value>
    // Path mode carries the VNode child-index sequence; devtools_update walks
    // the typed View<C::Msg> to the exact node and extracts its handler,
    // replacing the old Debug-substring heuristic.
    let mcp = iced::time::every(std::time::Duration::from_millis(16)).filter_map(|_| {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        if let Some(rx) = lock.as_mut() {
            match rx.try_recv() {
                Ok(action) => {
                    let value_str = action.value.unwrap_or_default();
                    let payload = match action.target {
                        crate::ui::mcp_server::ActionTarget::Event { widget, event } => {
                            format!("__mcp_action|{}.{}|{}", widget, event, value_str)
                        }
                        crate::ui::mcp_server::ActionTarget::Path { path } => {
                            let path_str = path
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            format!(
                                "__mcp_action_path|{}|{}|{}",
                                path_str, action.action, value_str
                            )
                        }
                        crate::ui::mcp_server::ActionTarget::Fixture { .. } => {
                            // VM-only test fixtures are rejected by the MCP
                            // tool before reaching the Rust DevTools path.
                            return None;
                        }
                    };
                    Some(WrapperMsg::<C>::Debug(payload))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    });
    let f12 = iced::event::listen_with(|event, _status, _window_id| {
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
            if matches!(
                key,
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)
            ) {
                // 014 内存哨兵冻结态:F12 = 退出进程(逃生门)。
                if crate::ui::mem_guard::is_frozen() {
                    // PLAN-765 T-03（纪要三 F-8）：逃生门落审计（码 42 可
                    // 区分;原路径零痕迹）。
                    auto_lang_core::vm::ffi::stdlib::exit_audit(42, "mem_guard_f12_escape");
                    std::process::exit(42);
                }
                return Some(WrapperMsg::<C>::Debug(DEBUG_TOGGLE_EVENT.to_string()));
            }
        }
        None
    });
    let win = iced::event::listen_with(|event, _status, _window_id| match event {
        iced::Event::Window(iced::window::Event::Resized(size)) => Some(WrapperMsg::<C>::Debug(
            format!("__window_resized|{}x{}", size.width, size.height),
        )),
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(
            WrapperMsg::<C>::Debug(format!("__mouse_moved|{},{}", position.x, position.y)),
        ),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
            Some(WrapperMsg::<C>::Debug("__mouse_released".to_string()))
        }
        _ => None,
    });
    // Forward ignored key presses. The mapper is deliberately non-capturing
    // (an Iced 0.14 requirement); component-specific lookup happens in
    // `devtools_update` through `key_message`.
    let rust_keys = iced::keyboard::listen().map(|event| {
        let event = iced::Event::Keyboard(event);
        let key = rust_component_key_string(&event).unwrap_or_default();
        WrapperMsg::<C>::Debug(format!("__autoui_key|{key}"))
    });
    iced::Subscription::batch(vec![inner, f12, win, mcp, rust_keys])
}

/// Plan 407: tick subscription using run_with (avoids generic map const check).
fn tick_subscription<C: Component + 'static>(
    interval: std::time::Duration,
) -> iced::Subscription<WrapperMsg<C>> {
    iced::Subscription::run_with(interval, |interval| {
        let interval = *interval;
        futures::stream::unfold((), move |_| async move {
            tokio::time::sleep(interval).await;
            Some((WrapperMsg::<C>::Debug("__tick__".to_string()), ()))
        })
    })
}

/// Run a rust-mode Component with the F12 DevTools layer (Plan 311).
///
/// Mirrors [`run_app`] but instantiates `DevToolsWrapper::<C>` so F12 opens the
/// inspector. Generated `main.rs` (render=rust) calls this instead of
/// [`run_app`].
pub fn run_app_devtools<C>() -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    // Check tick interval and initial fit size at startup
    let tick = C::default();
    let (tick_ms, tick_msg) = (tick.tick_interval_ms(), tick.tick_msg());
    let interval = tick_ms.map(|ms| std::time::Duration::from_millis(ms as u64));
    let initial_fit_size = if startup_window_fit() {
        extract_root_view_fit_size(&tick.view())
    } else {
        None
    };
    drop(tick);

    // PLAN-022 T-03:窗口尺寸面启动种子(dev 跑法无 Resized 事件时也有
    // 真值;见 devtools_update __window_resized 臂同源同步)。
    let seed = startup_window_size();
    crate::ui::style::theme::set_window_width(seed.width);
    crate::ui::style::theme::set_window_height(seed.height);
    if std::env::var("AUTO_MA_DBG")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        eprintln!(
            "[P22-SEED] {}x{} thread_now={:?}",
            seed.width,
            seed.height,
            std::thread::current().name()
        );
    }

    iced::application(
        DevToolsWrapper::<C>::default,
        devtools_update,
        devtools_view,
    )
    .subscription(|w| {
        let mut subs: Vec<iced::Subscription<WrapperMsg<C>>> = vec![devtools_subscription(w)];
        // Plan 407: add tick subscription. Cannot use Subscription::map in
        // generic code (const check fails for non-concrete C). Instead,
        // emit a periodic WrapperMsg::Debug("__tick__") via a 'static recipe.
        if let Some(ms) = w.inner.tick_interval_ms() {
            // Use a custom subscription that doesn't go through .map().
            // Recipe: a struct implementing Hash + recipe pattern.
            subs.push(tick_subscription::<C>(std::time::Duration::from_millis(
                ms as u64,
            )));
        }
        // PLAN-811 R3/R4: 媒体独立唤醒（同步入口装配；与 with-task 入口
        // 共用 media_wake_subscription——门控/节拍/空闲零消息语义一致）。
        subs.push(typed_media_wake_subscription::<C>());
        iced::Subscription::batch(subs)
    })
    .window(iced::window::Settings {
        size: initial_fit_size.unwrap_or_else(startup_window_size),
        level: startup_window_level(),
        position: startup_window_position().unwrap_or_default(),
        ..Default::default()
    })
    // Plan 411: pac window/title envs(AUTO_VM_WINDOW/AUTO_VM_TITLE)对
    // rust 轨同语义生效(VM 轨同款读取面);DevTools 面板不受影响。
    .title(|_: &DevToolsWrapper<C>| window_title(String::from("Auto Lang - Iced")))
    .font(INTER_FONT_REGULAR)
    .font(INTER_FONT_MEDIUM)
    .font(INTER_FONT_SEMIBOLD)
    .default_font(INTER_FONT)
    .run()
    .map_err(|e| e.into())
}

/// Run a rust-mode Component — built by a boot closure — with the F12 DevTools
// ---------------------------------------------------------------------------
// Plan 020 T-03 —— native 像素臂渲染宿主（Component 泛型，independent 臂）
// ---------------------------------------------------------------------------

/// native 像素臂消息面：App 消息 / 协议入站 / 截图回调 / tick。
/// 无 Debug 派生——`iced::window::Screenshot` 只有 Clone；iced 0.14 的
/// `MaybeDebug/MaybeClone` 界限下合法（debug feature 未开）。
#[derive(Clone)]
pub enum NativePixelsMsg<C: Component> {
    App(C::Msg),
    Proto(Box<crate::ui::desktop_protocol::message::ProtocolMsg>),
    Shot(iced::window::Screenshot),
    Tick,
}

/// native 像素臂宿主状态：native 组件 + 像素桥（boot 期从 launch slot 取，
/// Hello 握手在此发起——run_session Standalone 像素臂同序）。
struct NativePixelsHost<C: Component> {
    inner: C,
    bridge: Option<crate::ui::desktop_protocol::pixels::PixelsChild>,
}

impl<C: Component> NativePixelsHost<C> {
    fn from_launch(component: C) -> Self {
        let mut host = Self {
            inner: component,
            bridge: crate::ui::desktop_protocol::pixels::take_launch(),
        };
        // Hello 握手失败不装桥——管道已死，宿主空转至窗亡（与解释态臂
        // run_session Standalone 同边角语义）。
        if let Some(bridge) = host.bridge.as_mut() {
            if !bridge.start() {
                host.bridge = None;
            }
        }
        host
    }
}

/// 状态变更（App 消息/tick）后的截图发起——去重防抖在桥内
/// （`request_capture`：同轮只发一次 Task）。
/// 截图 Task：经 `window::latest()` 取当前窗（iced 0.14 无 Id::MAIN——
/// 单窗 application 的窗 id 运行期解析）再 screenshot。
fn native_pixels_shot_task<C>() -> iced::Task<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    // PLAN-659 T-03：size 二段跳守卫——零维窗（最小化/关窗竞态）不发起
    // screenshot（iced_wgpu offscreen 零维 create_texture panic 防线）。
    iced::window::latest().then(|id| match id {
        Some(win) => iced::window::size(win).then(move |size| {
            if size.width <= 0.0 || size.height <= 0.0 {
                iced::Task::none()
            } else {
                iced::window::screenshot(win).map(NativePixelsMsg::Shot)
            }
        }),
        None => iced::Task::none(),
    })
}

fn native_pixels_capture<C>(state: &mut NativePixelsHost<C>) -> iced::Task<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    let Some(bridge) = state.bridge.as_mut() else {
        return iced::Task::none();
    };
    if bridge.request_capture() {
        native_pixels_shot_task::<C>()
    } else {
        iced::Task::none()
    }
}

/// update：Proto/Shot 两臂镜像 run_session 的解释态像素臂
/// （DesktopEvent::PixelsProtocol/PixelsShot 同名臂）；App/Tick 臂驱动
/// `inner.on` 后发起截图。native v1 边界：`on_protocol` 组件参数传
/// None——StateSnapshot 注入 not-yet（Plan 020 §5.1）。
fn native_pixels_update<C>(
    state: &mut NativePixelsHost<C>,
    msg: NativePixelsMsg<C>,
) -> iced::Task<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    use crate::ui::desktop_protocol::message::ProtocolMsg;
    match msg {
        NativePixelsMsg::App(m) => {
            state.inner.on(m);
            native_pixels_capture(state)
        }
        NativePixelsMsg::Tick => {
            if let Some(m) = state.inner.tick_msg() {
                state.inner.on(m);
            }
            native_pixels_capture(state)
        }
        NativePixelsMsg::Proto(m) => {
            let Some(mut bridge) = state.bridge.take() else {
                return iced::Task::none();
            };
            let (replies, want_capture) = bridge.on_protocol(*m, None);
            for reply in replies {
                if !bridge.send(&reply) {
                    state.bridge = Some(bridge);
                    return iced::exit();
                }
            }
            let mut tasks = Vec::new();
            if want_capture && bridge.request_capture() {
                tasks.push(native_pixels_shot_task::<C>());
            }
            let detached = bridge.is_detached();
            state.bridge = Some(bridge);
            if detached {
                return iced::exit();
            }
            iced::Task::batch(tasks)
        }
        NativePixelsMsg::Shot(ss) => {
            let Some(bridge) = state.bridge.as_mut() else {
                return iced::Task::none();
            };
            // 物理像素 → 表面逻辑尺寸（盒降采样）：shm 槽按逻辑尺寸定档，
            // HiDPI scale 经此对齐（497 快照同型）。
            let (lw, lh) = bridge.size();
            let lw = lw.max(1.0).ceil() as u32;
            let lh = lh.max(1.0).ceil() as u32;
            let rgba = crate::ui::desktop_protocol::pixels::downsample_to_logical(
                ss.rgba.as_ref(),
                ss.size.width,
                ss.size.height,
                lw,
                lh,
            );
            let frame = crate::ui::desktop_protocol::pixels::PixelsFrame {
                rgba,
                w: lw,
                h: lh,
                stride: lw * 4,
            };
            if let Some(frame_msg) = bridge.capture(frame) {
                if !bridge.send(&ProtocolMsg::Frame(frame_msg)) {
                    return iced::exit();
                }
            }
            if bridge.is_detached() {
                return iced::exit();
            }
            iced::Task::none()
        }
    }
}

/// view：native 组件 View → iced（`IntoIcedElement`），消息面映射到
/// `NativePixelsMsg::App`。
fn native_pixels_view<C>(state: &NativePixelsHost<C>) -> iced::Element<'_, NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    let mut view = state.inner.view();
    rewrite_viewport_units(&mut view);
    view.into_iced().map(NativePixelsMsg::App)
}

/// 协议入站轮询（pixels 协议订阅同型——该订阅 Output 钉死
/// DesktopMessage，native 臂消息面独立建 recipe；空拍 5ms 重试）。
fn native_pixels_proto_subscription<C>() -> iced::Subscription<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    use iced_futures::subscription::Recipe;

    struct NativePixelsProtoRecipe<C: Component>(std::marker::PhantomData<fn() -> C>);

    impl<C: Component> std::hash::Hash for NativePixelsProtoRecipe<C> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            "auto-lang-native-pixels-protocol".hash(state);
        }
    }

    impl<C: Component> Recipe for NativePixelsProtoRecipe<C>
    where
        C: 'static,
    {
        type Output = NativePixelsMsg<C>;

        fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
            std::hash::Hash::hash(self, state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced_futures::subscription::EventStream,
        ) -> iced_futures::BoxStream<Self::Output> {
            use iced_futures::futures::stream::{unfold, StreamExt};
            unfold((), |()| async move {
                loop {
                    match crate::ui::desktop_protocol::pixels::poll_transport() {
                        Some(Ok(msg)) => {
                            return Some((NativePixelsMsg::Proto(Box::new(msg)), ()));
                        }
                        Some(Err(_codec)) => continue,
                        None => {
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        }
                    }
                }
            })
            .boxed()
        }
    }

    iced_futures::subscription::from_recipe(NativePixelsProtoRecipe(std::marker::PhantomData))
}

/// tick 订阅（run_app_devtools 的 tick_subscription 同型——.map() 在泛型
/// 代码不可用，'static recipe 直发 Tick 变体；interval 入 hash 防漂重订阅）。
fn native_pixels_tick_subscription<C>(
    interval: std::time::Duration,
) -> iced::Subscription<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    use iced_futures::subscription::Recipe;

    struct NativePixelsTickRecipe<C: Component> {
        interval: std::time::Duration,
        _c: std::marker::PhantomData<fn() -> C>,
    }

    impl<C: Component> std::hash::Hash for NativePixelsTickRecipe<C> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            ("auto-lang-native-pixels-tick", self.interval).hash(state);
        }
    }

    impl<C: Component> Recipe for NativePixelsTickRecipe<C>
    where
        C: 'static,
    {
        type Output = NativePixelsMsg<C>;

        fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
            std::hash::Hash::hash(self, state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced_futures::subscription::EventStream,
        ) -> iced_futures::BoxStream<Self::Output> {
            use iced_futures::futures::stream::{unfold, StreamExt};
            let interval = self.interval;
            unfold((), move |()| {
                let interval = interval;
                async move {
                    tokio::time::sleep(interval).await;
                    Some((NativePixelsMsg::<C>::Tick, ()))
                }
            })
            .boxed()
        }
    }

    iced_futures::subscription::from_recipe(NativePixelsTickRecipe {
        interval,
        _c: std::marker::PhantomData,
    })
}

fn native_pixels_subscription<C>(
    state: &NativePixelsHost<C>,
) -> iced::Subscription<NativePixelsMsg<C>>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    let mut subs = vec![native_pixels_proto_subscription::<C>()];
    if state.inner.tick_msg().is_some() {
        if let Some(ms) = state.inner.tick_interval_ms() {
            subs.push(native_pixels_tick_subscription::<C>(
                std::time::Duration::from_millis(ms as u64),
            ));
        }
    }
    iced::Subscription::batch(subs)
}

/// Plan 020 T-03 —— native 像素臂渲染宿主入口：隐藏单窗 iced application
/// over `C: Component` + 像素桥（协议轮询订阅 + 截图回发 + Close 退出）。
/// 窗尺寸 = 表面逻辑尺寸（shm 槽/截图降采样口径同源）；`visible: false`
/// ——渲染/截图照常，不进 OS 桌面 z 序（run_session Standalone+pixels
/// 同语义，合成归宿主虚拟窗）。
pub fn run_native_iced_pixels<C>(component: C, width: f32, height: f32) -> AppResult<()>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    // 单发组件装载（RefCell 内变——new 闭包需 Fn，生成 main 的 __init
    // 同型手法）。
    let cell = std::cell::RefCell::new(Some(component));
    iced::application(
        move || {
            let component = cell.borrow_mut().take().expect("pixels host init once");
            NativePixelsHost::from_launch(component)
        },
        native_pixels_update::<C>,
        native_pixels_view::<C>,
    )
    .subscription(native_pixels_subscription::<C>)
    .window(iced::window::Settings {
        size: iced::Size::new(width, height),
        position: iced::window::Position::Specific(iced::Point::new(80.0, 80.0)),
        visible: false,
        ..Default::default()
    })
    .title(|_: &NativePixelsHost<C>| window_title(String::from("Auto Lang - Pixels")))
    .font(INTER_FONT_REGULAR)
    .font(INTER_FONT_MEDIUM)
    .font(INTER_FONT_SEMIBOLD)
    .default_font(INTER_FONT)
    .run()
    .map_err(|e| e.into())
}

/// layer (Plan 311 P2-A). The async-init counterpart of [`run_app_devtools`]:
/// covers apps whose `main.rs` codegens to `run_app_with_task` (e.g. any app
/// with an `__InitLoaded` init API, like `015-notes`).
///
/// Unlike [`run_app_devtools`], this does NOT require `C: Default` — the boot
/// closure creates the state (enabling async initialization patterns). The
/// boot `Task<C::Msg>` is lifted to `Task<WrapperMsg<C>>` so the initial
/// background load still fires.
pub fn run_app_with_task_devtools<C>(
    boot: impl Fn() -> (C, iced::Task<C::Msg>) + 'static,
) -> AppResult<()>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    iced::application(
        move || {
            let (inner, task) = boot();
            (
                DevToolsWrapper::from_inner(inner),
                task.map(WrapperMsg::<C>::Inner),
            )
        },
        devtools_update,
        devtools_view,
    )
    .subscription(move |w| {
        let mut subs: Vec<iced::Subscription<WrapperMsg<C>>> = vec![devtools_subscription(w)];
        // PLAN-811 R4: async（with-task）启动入口与同步入口共用同一媒体
        // 唤醒装配——async-init 应用同样不依赖应用 timer 收到事件。
        subs.push(typed_media_wake_subscription::<C>());
        // Plan 407 + 014 直键入:with-task 启动变体也挂 tick 订阅
        // (run_app_devtools 同款;此前 async init 应用无 tick——at-app 的
        // timer 驱动流式刷新/banner 收割全靠它,无按钮后没有别的 tick 源)。
        if let Some(ms) = w.inner.tick_interval_ms() {
            if w.inner.tick_msg().is_some() {
                subs.push(tick_subscription::<C>(std::time::Duration::from_millis(
                    ms as u64,
                )));
            }
        }
        iced::Subscription::batch(subs)
    })
    .window(iced::window::Settings {
        size: startup_window_size(),
        level: startup_window_level(),
        position: startup_window_position().unwrap_or_default(),
        ..Default::default()
    })
    // Plan 411: pac window/title envs 对 rust 轨同语义生效(上方
    // run_app_devtools 同款;原 with-task 变体硬编码 1600×900 且无标题)。
    .title(|_: &DevToolsWrapper<C>| window_title(String::from("Auto Lang - Iced")))
    // Plan 411 P1-C: 内嵌 Inter 三字重 + 默认 family(中文字形回退系统)。
    .font(INTER_FONT_REGULAR)
    .font(INTER_FONT_MEDIUM)
    .font(INTER_FONT_SEMIBOLD)
    .default_font(INTER_FONT)
    .run()
    .map_err(|e| e.into())
}

// ---------------------------------------------------------------------
// rust-mode panel renderers (emit `WrapperMsg<C>`; MVP, tree-selection only)
// ---------------------------------------------------------------------

/// Recursive element-tree renderer for the live VTree (rust mode). Mirrors
/// [`render_vtree_into`] but emits `WrapperMsg::Debug("__vnode_select_<id>")`.
fn rdt_render_vtree<C: Component + 'static>(
    tree: &crate::ui::vnode::VTree,
    node: &crate::ui::vnode::VNode,
    depth: usize,
    selected: &Option<crate::ui::vnode::VNodeId>,
    rows: &mut Vec<iced::Element<'static, WrapperMsg<C>>>,
) {
    let indent = "  ".repeat(depth);
    let is_selected = *selected == Some(node.id);
    let has_children = !node.children.is_empty();
    let prefix = if has_children { "▼ " } else { "  " };
    let summary = vnode_summary(node);
    let label = if summary.is_empty() {
        format!("{}{}{}", indent, prefix, node.kind)
    } else {
        format!("{}{}{} {}", indent, prefix, node.kind, summary)
    };
    let text_color = if is_selected {
        iced::Color::from_rgb(0.85, 0.4, 0.1)
    } else if has_children {
        iced::Color::from_rgb(0.2, 0.4, 0.7)
    } else {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    };

    let click_area = mouse_area(
        container(text(label).size(10).color(text_color))
            .style(move |_: &iced::Theme| {
                if is_selected {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.95, 0.85, 0.7, 0.6,
                        ))),
                        ..Default::default()
                    }
                } else {
                    container::Style::default()
                }
            })
            .padding(iced::Padding::new(2.0)),
    )
    .on_press(WrapperMsg::<C>::Debug(format!(
        "{}{}",
        DEBUG_SELECT_VNODE_PREFIX,
        node.id.as_u64()
    )));
    rows.push(click_area.into());

    if let Some(children) = tree.children(node.id) {
        for child in children {
            rdt_render_vtree::<C>(tree, child, depth + 1, selected, rows);
        }
    }
}

/// Left pane: the live element tree.
fn rdt_elements_tab<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    let vtree = dt.live_vtree.borrow().clone();
    let selected = dt.selected_vnode.borrow().clone();
    match vtree {
        Some(tree) => {
            let mut rows: Vec<iced::Element<'static, WrapperMsg<C>>> = Vec::new();
            if let Some(root) = tree.root() {
                rdt_render_vtree::<C>(&tree, root, 0, &selected, &mut rows);
            }
            let mut col = column![].spacing(1);
            for r in rows {
                col = col.push(r);
            }
            col.into()
        }
        None => column![text("组件树不可用")
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5))]
        .into(),
    }
}

/// Helper: clone the selected `VNode` out of the live VTree, or run `on_missing`.
fn rdt_with_selected<C: Component + 'static, F>(
    dt: &DevToolsState,
    on_missing: &str,
    f: F,
) -> iced::Element<'static, WrapperMsg<C>>
where
    F: FnOnce(&crate::ui::vnode::VNode) -> iced::Element<'static, WrapperMsg<C>>,
{
    let vtree = dt.live_vtree.borrow().clone();
    let selected = dt.selected_vnode.borrow().clone();
    match (vtree, selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => f(node),
            None => placeholder_panel::<WrapperMsg<C>>(on_missing),
        },
        _ => placeholder_panel::<WrapperMsg<C>>(on_missing),
    }
}

/// Title row for the right pane: the selected node's kind + a hint.
fn rdt_selected_title<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    let selected = dt.selected_vnode.borrow().clone();
    let vtree = dt.live_vtree.borrow().clone();
    let label = match (vtree.as_ref(), selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => {
                if !node.label.is_empty() {
                    node.label.clone()
                } else {
                    format!("{:?}", node.kind)
                }
            }
            None => "无选中元素".to_string(),
        },
        _ => "无选中元素 — 点击左侧元素树".to_string(),
    };
    text(label)
        .size(11)
        .color(iced::Color::from_rgb(0.2, 0.2, 0.2))
        .into()
}

/// 盒模型 section body (rust mode): declared inset diagram, or pending.
fn rdt_layout_section<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let cache = dt.live_cache.borrow().clone();
        let Some(cache) = cache else {
            return layout_pending_panel::<WrapperMsg<C>>();
        };
        let Some(computed) = cache.get(node.id) else {
            return layout_pending_panel::<WrapperMsg<C>>();
        };
        let bm = match &computed.box_model {
            Some(bm) => bm.clone(),
            None => {
                // No declared insets: show a minimal note instead of an empty
                // diagram.
                return text("(本节点无声明内边距)")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    .into();
            }
        };
        let mut col = column![].spacing(4);
        col = col.push(render_box_model_diagram::<WrapperMsg<C>>(&bm));
        // Declared insets summary (measured content rect deferred — MVP).
        col = col.push(kv_row::<WrapperMsg<C>>(
            "content",
            "(尺寸待测量)".to_string(),
        ));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "padding",
            format_insets(&bm.padding),
        ));
        col = col.push(kv_row::<WrapperMsg<C>>("border", format_insets(&bm.border)));
        col = col.push(kv_row::<WrapperMsg<C>>("margin", format_insets(&bm.margin)));
        col.into()
    })
}

/// Computed section body (rust mode): layout props + computed style k/v.
fn rdt_computed_section<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let mut col = column![].spacing(3);
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_x", center_x.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_y", center_y.to_string()));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
            }
            _ => {}
        }
        let cache = dt.live_cache.borrow().clone();
        let mut have = false;
        if let Some(cache) = cache {
            if let Some(computed) = cache.get(node.id) {
                for (k, v) in &computed.computed_style {
                    col = col.push(kv_row::<WrapperMsg<C>>(k.as_str(), v.clone()));
                    have = true;
                }
            }
        }
        if !have {
            col = col.push(
                text("(无 computed 样式)")
                    .size(9)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
        col.into()
    })
}

/// Properties section body (rust mode): the selected VNode's props fields.
fn rdt_props_section<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let mut col = column![].spacing(3);
        col = col.push(kv_row::<WrapperMsg<C>>("kind", format!("{:?}", node.kind)));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "path",
            format!(
                "[{}]",
                node.path
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Empty => {}
            VNodeProps::Text { content, .. } => {
                col = col.push(kv_row::<WrapperMsg<C>>("content", content.clone()))
            }
            VNodeProps::Button { label, .. } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()))
            }
            VNodeProps::Input {
                placeholder,
                value,
                password,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("placeholder", placeholder.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("value", value.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("password", password.to_string()));
            }
            VNodeProps::Textarea { placeholder, value } => {
                col = col.push(kv_row::<WrapperMsg<C>>("placeholder", placeholder.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("value", value.clone()));
            }
            VNodeProps::Checkbox { label, is_checked } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "is_checked",
                    is_checked.to_string(),
                ));
            }
            VNodeProps::Radio { label, is_selected } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "is_selected",
                    is_selected.to_string(),
                ));
            }
            VNodeProps::Select {
                options,
                selected_index,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "options",
                    format!("[{}]", options.join(", ")),
                ));
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "selected_index",
                    format!("{:?}", selected_index),
                ));
            }
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_x", center_x.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_y", center_y.to_string()));
            }
            VNodeProps::Scrollable { .. } => {}
            VNodeProps::Slider {
                min,
                max,
                value,
                step,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("min", format!("{}", min)));
                col = col.push(kv_row::<WrapperMsg<C>>("max", format!("{}", max)));
                col = col.push(kv_row::<WrapperMsg<C>>("value", format!("{}", value)));
                col = col.push(kv_row::<WrapperMsg<C>>("step", format!("{:?}", step)));
            }
            VNodeProps::ProgressBar { progress } => {
                col = col.push(kv_row::<WrapperMsg<C>>("progress", format!("{}", progress)))
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()))
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
                ..
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "col_spacing",
                    col_spacing.to_string(),
                ));
            }
        }
        col.into()
    })
}

/// One collapsible section (rust mode). Header click emits
/// `__inspector_section_<tail>`.
fn rdt_collapsible_section<C: Component + 'static>(
    title: &'static str,
    collapsed: bool,
    tail: &str,
    body: iced::Element<'static, WrapperMsg<C>>,
) -> iced::Element<'static, WrapperMsg<C>> {
    let marker = if collapsed { "▸" } else { "▾" };
    let header = mouse_area(
        row![
            text(marker).size(10),
            text(title)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.8)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .on_press(WrapperMsg::<C>::Debug(format!(
        "{}{}",
        DEBUG_INSPECTOR_SECTION_PREFIX, tail
    )));

    let mut col = column![]
        .spacing(3)
        .push(container(header).padding([2.0, 4.0]));
    if !collapsed {
        col = col.push(body);
    }
    col.into()
}

/// Right pane: selected-node title + three collapsible sections.
fn rdt_inspector<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    let secs = *dt.inspector_sections.borrow();
    let mut col = column![].spacing(6);
    col = col.push(rdt_selected_title::<C>(dt));
    col = col.push(rdt_collapsible_section::<C>(
        "盒模型 Box Model",
        secs.box_collapsed,
        "box",
        rdt_layout_section::<C>(dt),
    ));
    col = col.push(rdt_collapsible_section::<C>(
        "Computed",
        secs.computed_collapsed,
        "computed",
        rdt_computed_section::<C>(dt),
    ));
    col = col.push(rdt_collapsible_section::<C>(
        "Properties",
        secs.props_collapsed,
        "props",
        rdt_props_section::<C>(dt),
    ));
    col.into()
}

/// Full DevTools panel: header + [element tree | divider | inspector].
fn rdt_devtools_panel<C: Component + 'static>(
    dt: &DevToolsState,
) -> iced::Element<'static, WrapperMsg<C>> {
    let close_btn = container(
        mouse_area(
            text("✕")
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .on_press(WrapperMsg::<C>::Debug("__close_devtools".to_string())),
    )
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.95, 0.95, 0.95,
        ))),
        border: iced::Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding(iced::Padding::new(4.0));
    let header = row![
        text("DevTools · rust").size(11),
        text("(F12 关闭)")
            .size(9)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        close_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .width(iced::Length::Fill);

    let ratio = (*dt.inspector_split_ratio.borrow()).clamp(0.1, 0.9);
    let tree_pane = scrollable(rdt_elements_tab::<C>(dt))
        .width(iced::Length::FillPortion((ratio * 1000.0) as u16))
        .height(iced::Length::Fill);
    let inspector_pane = scrollable(rdt_inspector::<C>(dt))
        .width(iced::Length::FillPortion(((1.0 - ratio) * 1000.0) as u16))
        .height(iced::Length::Fill);
    let divider = mouse_area(
        container(iced::widget::Space::new().width(6))
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.82, 0.82, 0.82,
                ))),
                ..Default::default()
            })
            .width(6)
            .height(iced::Length::Fill),
    )
    .on_press(WrapperMsg::<C>::Debug("__inner_divider_press".to_string()));
    let content = row![tree_pane, divider, inspector_pane]
        .spacing(0)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

    let panel_width = *dt.devtools_panel_width.borrow();
    let panel_col = column![header, content]
        .spacing(4)
        .width(panel_width)
        .height(iced::Length::Fill);

    container(panel_col)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.98, 0.98, 0.98,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding(iced::Padding::new(6.0))
        .width(panel_width)
        .height(iced::Length::Fill)
        .into()
}

/// Format a debug `EdgeInsets` as `t / r / b / l`.
fn format_insets(ei: &crate::ui::debug::EdgeInsets) -> String {
    format!(
        "{:.0} / {:.0} / {:.0} / {:.0}",
        ei.top, ei.right, ei.bottom, ei.left
    )
}

