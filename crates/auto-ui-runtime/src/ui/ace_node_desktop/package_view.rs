use ace_av::age_package::{WidgetElement, WidgetViewItem};

use crate::ui::ace_svg;
use crate::ui::aura_view_builder::parse_coords_extent;
use crate::ui::view::PointerMoveHandler;
use crate::ui::ace_package_bridge::PenPhase;
use crate::ui::aura_view_builder::{rederive_layout, RederivedLayout};
use crate::ui::iced::image_surface_input::ImageSurfaceCallback;
use crate::ui::style::{SizeValue, StyleClass};
use crate::ui::view::{PopoverAnchor, PopoverPlacement};
use crate::ui::{Style, View};

use super::{package_images::{PackageImages, SurfaceCallbacks}, NodeDesktopMsg, PackageFrame};
use std::sync::Arc;

pub(super) fn view(
    session: i32,
    instance: usize,
    frame: &PackageFrame,
    images: Option<&PackageImages>,
    outlet: Option<&View<NodeDesktopMsg>>,
    routed: bool,
    dialogs: &mut Vec<View<NodeDesktopMsg>>,
) -> View<NodeDesktopMsg> {
    let mut context = FrameView {
        session,
        instance,
        frame,
        images,
        routed,
        dialogs,
    };
    let mut builder = View::col()
        .children(context.items(&frame.view, &mut Vec::new(), outlet))
        .spacing(4);
    if let Some(images) = images {
        images.finish_frame(instance, frame.revision);
    }
    if owns_viewport(frame) {
        let style = if routed && outlet.is_some() {
            "w-full h-full bg-background text-foreground"
        } else {
            "w-full h-full"
        };
        builder = builder.style(style);
    }
    builder.build()
}

/// A fullscreen package provides its own scroll regions.
pub(super) fn owns_viewport(frame: &PackageFrame) -> bool {
    let mut roots = frame.view.iter().filter(|item| !matches!(item, WidgetViewItem::Timer(_)));
    let (Some(WidgetViewItem::Element(root)), None) = (roots.next(), roots.next()) else {
        return false;
    };
    style(root).is_some_and(|style| {
        style
            .classes
            .iter()
            .rev()
            .find_map(|class| match class {
                StyleClass::Height(size) => Some(matches!(size, SizeValue::Full | SizeValue::Screen)),
                _ => None,
            })
            .unwrap_or(false)
    })
}

pub(super) fn focus_id(session: i32, instance: usize, target: &str) -> String {
    format!("ace-package-focus/{session}/{instance}/{target}")
}

struct FrameView<'a> {
    session: i32,
    instance: usize,
    frame: &'a PackageFrame,
    images: Option<&'a PackageImages>,
    routed: bool,
    dialogs: &'a mut Vec<View<NodeDesktopMsg>>,
}

impl FrameView<'_> {
    fn items(
        &mut self,
        items: &[WidgetViewItem],
        path: &mut Vec<usize>,
        outlet: Option<&View<NodeDesktopMsg>>,
    ) -> Vec<View<NodeDesktopMsg>> {
        items
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| match item {
                WidgetViewItem::Text(text) => Some(View::text(text.clone())),
                WidgetViewItem::Element(element) => {
                    path.push(slot);
                    let rendered = self.element(element, path, outlet);
                    path.pop();
                    Some(rendered)
                }
                WidgetViewItem::Timer(_) => None,
            })
            .collect()
    }

    fn element(
        &mut self,
        element: &WidgetElement,
        path: &mut Vec<usize>,
        outlet: Option<&View<NodeDesktopMsg>>,
    ) -> View<NodeDesktopMsg> {
        if self.frame.is_widget(path) {
            return View::col().children(self.items(&element.children, path, None)).build();
        }
        let style = style(element);
        if style.as_ref().is_some_and(Style::is_hidden) {
            return View::empty();
        }
        if element.tag == "outlet" {
            return outlet.cloned().unwrap_or_else(View::empty);
        }
        if element.tag == "svg" {
            return match ace_svg::checked_document(element) {
                Ok(document) => View::Image {
                    src: format!("svgdoc:{document}"),
                    style,
                },
                Err(error) => View::text(error),
            };
        }
        if matches!(element.tag.as_str(), "drawer" | "alert-dialog")
            && prop(element, "open") != Some("true")
        {
            return View::empty();
        }
        if element.tag == "input" {
            return self.input(element, path);
        }
        let mut children = self.items(&element.children, path, outlet);
        if element.tag == "alert-dialog" {
            // A modal's viewport is the window, even when its owner scrolls.
            self.dialogs.push(View::Popover {
                anchor: PopoverAnchor::Widget(Box::new(View::empty())),
                content: Box::new(View::col().children(children).build()),
                placement: PopoverPlacement::Modal,
                open: true,
                on_dismiss: None,
            });
            return View::empty();
        }
        let label = element.label.as_deref().or_else(|| prop(element, "text"));
        let link = self.routed.then(|| self.frame.link(path)).flatten();
        let click = link.or_else(|| self.frame.click(path)).map(|target| self.action(target));
        let control = self.frame.control(path);
        let context_menu = control.and_then(|control| control.context_menu).map(|target| self.action(target));
        let double_click = control.and_then(|control| control.double_click).map(|target| self.action(target));
        if element.tag == "checkbox" {
            return View::Checkbox {
                is_checked: prop(element, "checked") == Some("true"),
                label: label.unwrap_or_default().to_owned(),
                on_toggle: click,
                style,
            };
        }
        if matches!(element.tag.as_str(), "button" | "alert-dialog-action" | "alert-dialog-cancel")
            || link.is_some()
        {
            if let Some(onclick) = click {
                let icon = prop(element, "icon");
                let label = label.or_else(|| first_text(&element.children))
                    .filter(|label| !label.is_empty()).or(icon)
                    .or_else(|| prop(element, "aria-label")).unwrap_or_default();
                if let Some(icon) = icon {
                    children.insert(0, View::Image {
                        src: format!("lucide:{icon}"),
                        style: Style::parse("w-4 h-4").ok(),
                    });
                }
                let content = (!children.is_empty()).then(|| {
                    let builder = match rederive_layout(style.as_ref()) {
                        Some(RederivedLayout::Column) => View::col(),
                        _ => View::row(),
                    };
                    Box::new(builder.spacing(6).children(children).build())
                });
                let button = View::Button {
                    label: label.to_owned(),
                    onclick: onclick.clone(),
                    style,
                    on_right_click: context_menu,
                    content,
                    disabled: prop(element, "disabled") == Some("true"),
                    keyboard: None,
                };
                let button = pointer_events(button, None, double_click, None);
                return self.focusable(button, path, Some(onclick));
            }
        }
        if element.tag == "drawer" {
            if let Some(onclick) = click {
                children.push(View::button_styled("Close drawer", onclick, "px-2 py-1"));
            }
            return self.focusable(View::col().children(children).build(), path, None);
        }
        if let Some(pointer) = self.frame.pointer(path) {
            let logical_extent = prop(element, "coords").and_then(parse_coords_extent)
                .filter(|(width, height)| width.is_finite() && height.is_finite());
            if pointer.motion.is_some() && logical_extent.is_none() && prop(element, "coords").is_some() {
                return View::text("ace.desktop.pointer: coords must have a finite WxH extent");
            }
            let (session, instance, revision) = (self.session, self.instance, self.frame.revision);
            return View::MouseArea {
                content: Box::new(View::col().children(children).build()),
                on_click: click,
                on_enter: pointer.enter.map(|target| self.action(target)),
                on_exit: pointer.leave.map(|target| self.action(target)),
                on_double_click: double_click,
                on_context_menu: context_menu,
                on_release: None,
                on_move: pointer.motion.map(|target| {
                    PointerMoveHandler::new(move |x, y| NodeDesktopMsg::PackagePointer {
                        session, instance, revision, target, x, y,
                    })
                }),
                logical_extent,
                style,
            };
        }
        let content = match element.tag.as_str() {
            _ if self.frame.image_surface(path).is_some() => self.image_surface(element, path, style),
            // PLAN-118：canvas 场景件 —— 包面画布臂。场景数据通道（列表态读面）
            // 随 PLAN-117 live_session 链落主后接通；当前渲染空场景，tap 命中
            // 通路（onhit → PackageText 带命中 id）已完整接线。
            "canvas" => {
                let logical_extent =
                    prop(element, "coords").and_then(parse_coords_extent);
                let clear = prop(element, "clear").map(str::to_owned);
                let on_hit = self.frame.hit(path).map(|target| {
                    let (session, instance, revision) =
                        (self.session, self.instance, self.frame.revision);
                    crate::ui::view::ElementHitHandler::new(move |hit_id| {
                        NodeDesktopMsg::PackageText {
                            session,
                            instance,
                            revision,
                            target,
                            text: hit_id,
                        }
                    })
                });
                let pen = self.frame.pen(path).unwrap_or((None, None, None));
                let pen_handler = |target: Option<usize>, phase: PenPhase| {
                    let (session, instance, revision) =
                        (self.session, self.instance, self.frame.revision);
                    target.map(move |target| {
                        PointerMoveHandler::new(move |x, y| NodeDesktopMsg::PackagePen {
                            session,
                            instance,
                            revision,
                            target,
                            x,
                            y,
                            phase,
                        })
                    })
                };
                return View::Canvas {
                    scene: crate::ui::view::CanvasScene {
                        strokes: Vec::new(),
                        nodes: Vec::new(),
                        edges: Vec::new(),
                        labels: Vec::new(),
                    },
                    logical_extent,
                    clear,
                    on_pen_start: pen_handler(pen.0, PenPhase::Start),
                    on_pen_move: pen_handler(pen.1, PenPhase::Move),
                    on_pen_end: pen_handler(pen.2, PenPhase::End),
                    on_hit,
                    style,
                };
            }
            "icon" => View::Image {
                src: format!("lucide:{}", prop(element, "name").unwrap_or_default()),
                style,
            },
            "text" | "span" | "h1" | "h2" | "h3" | "th" | "td" | "codeblock"
            | "alert-dialog-title" | "alert-dialog-description"
                if children.is_empty() =>
            {
                View::Text {
                    content: label
                        .or_else(|| prop(element, "code"))
                        .unwrap_or_default()
                        .to_owned(),
                    style,
                    selectable: element.tag == "codeblock",
                }
            }
            "sidebar_content" => {
                let content = View::col().children(children).build();
                let mut viewport = View::scrollable(content);
                if let Some(style) = style {
                    viewport = viewport.with_style(style);
                }
                viewport.build()
            }
            tag => {
                if let Some(label) = label {
                    children.insert(0, View::text(label.to_owned()));
                }
                let layout = rederive_layout(style.as_ref()).unwrap_or(match tag {
                    "row" | "tr" | "mouse-area" | "alert-dialog-footer" => RederivedLayout::Row,
                    "grid" => RederivedLayout::Grid { cols: 1, gap: 0 },
                    _ => RederivedLayout::Column,
                });
                layout_view(tag, layout, style, children)
            }
        };
        let content = pointer_events(content, click.clone(), double_click, context_menu);
        self.focusable(content, path, click)
    }

    fn image_surface(&self, element: &WidgetElement, path: &[usize], style: Option<Style>) -> View<NodeDesktopMsg> {
        let bindings = self.frame.image_surface(path).expect("published image surface");
        let source = prop(element, "src").unwrap_or_default();
        let private = crate::back_proxy::native_files::is_media(source);
        let callbacks = || SurfaceCallbacks {
            revision: self.frame.revision,
            loaded: bindings.loaded.and_then(|target| self.frame.queued_action(target)),
            error: bindings.error.and_then(|target| self.frame.queued_action(target)),
        };
        let src = if private {
            if let Some(images) = self.images {
                images.project_private(self.instance, path, source, callbacks());
            }
            source.to_owned()
        } else if source.starts_with("/api/__auto/media/") {
            source.to_owned()
        } else if let Some(images) = self.images {
            if let Some(url) = images.http_url(source) {
                images.project_http(
                    self.instance,
                    path,
                    url,
                    prop(element, "cache_identity").unwrap_or_default(),
                    callbacks(),
                )
            } else {
                images.public_source(source).unwrap_or_default()
            }
        } else {
            String::new()
        };
        let number = |key, default| {
            prop(element, key).and_then(|value| value.parse::<f32>().ok())
                .filter(|value| value.is_finite()).unwrap_or(default)
        };
        let text = |key, default| {
            prop(element, key).filter(|value| !value.is_empty()).unwrap_or(default).to_owned()
        };
        let zoom = number("zoom", 1.0);
        let (session, instance, revision) = (self.session, self.instance, self.frame.revision);
        let input = |target: usize| {
            let has_payload = self.frame.actions()[target].source.image_surface_payload.is_some();
            ImageSurfaceCallback::from_closure(Arc::new(move |carry| {
                if has_payload {
                    NodeDesktopMsg::PackageImageSurface { session, instance, revision, target, carry }
                } else {
                    NodeDesktopMsg::PackageClick { session, instance, revision, target }
                }
            }))
        };
        View::ImageSurface {
            src,
            alt: text("alt", ""),
            width: number("width", 0.0) as u32,
            height: number("height", 0.0) as u32,
            quality: number("quality", 90.0).clamp(0.0, 100.0) as u8,
            fit: text("fit", "contain"),
            zoom: if zoom > 0.0 { zoom } else { 1.0 },
            offset_x: number("offset_x", 0.0),
            offset_y: number("offset_y", 0.0),
            rotation: (number("rotation", 0.0) as i32).rem_euclid(360),
            filter: text("filter", "high"),
            on_error: (!private).then(|| bindings.error.map(|target| self.action(target))).flatten(),
            on_loaded: (!private).then(|| bindings.loaded.map(|target| self.action(target))).flatten(),
            on_wheel: bindings.wheel.map(input),
            on_pan: bindings.pan.map(input),
            on_double_click: bindings.double_click.map(input),
            style,
        }
    }

    fn input(&self, element: &WidgetElement, path: &[usize]) -> View<NodeDesktopMsg> {
        let mut input = View::input(prop(element, "placeholder").unwrap_or_default())
            .value(prop(element, "value").unwrap_or_default())
            .disabled(prop(element, "disabled") == Some("true"));
        if let Some(style) = style(element) {
            input = input.with_style(style);
        }
        if prop(element, "type") == Some("password") {
            input = input.password();
        }
        if let Some(control) = self.frame.control(path) {
            if let Some(target) = control.input {
                let (session, instance, revision) = (self.session, self.instance, self.frame.revision);
                input = input.on_text(move |text| NodeDesktopMsg::PackageText {
                    session, instance, revision, target, text,
                });
            }
            if let Some(target) = control.submit {
                input = input.on_submit(self.action(target));
            }
        }
        self.focusable(input.build(), path, None)
    }

    fn action(&self, target: usize) -> NodeDesktopMsg {
        NodeDesktopMsg::PackageClick {
            session: self.session,
            instance: self.instance,
            revision: self.frame.revision,
            target,
        }
    }

    fn focusable(&self, child: View<NodeDesktopMsg>, path: &[usize], activation: Option<NodeDesktopMsg>) -> View<NodeDesktopMsg> {
        let Some(control) = self.frame.control(path) else { return child };
        let Some(id) = &control.focus else { return child };
        View::FocusTarget {
            id: focus_id(self.session, self.instance, id),
            activation,
            keys: control.keys.iter().map(|(key, target)| (key.clone(), self.action(*target))).collect(),
            on_focus: control.on_focus.then(|| self.notice(id, true)),
            on_blur: control.on_blur.then(|| self.notice(id, false)),
            child: Box::new(child),
        }
    }

    fn notice(&self, id: &str, gained: bool) -> NodeDesktopMsg {
        NodeDesktopMsg::PackageFocus {
            session: self.session,
            instance: self.instance,
            revision: self.frame.revision,
            control: id.to_owned(),
            gained,
        }
    }
}

fn layout_view(
    tag: &str,
    layout: RederivedLayout,
    style: Option<Style>,
    children: Vec<View<NodeDesktopMsg>>,
) -> View<NodeDesktopMsg> {
    let explicit_scroll = matches!(tag, "scroll" | "sidebar_content");
    let style = style.map(|mut style| {
        if explicit_scroll
            && !style
                .classes
                .iter()
                .any(|class| matches!(class, StyleClass::OverflowAuto | StyleClass::OverflowYAuto))
        {
            style.classes.push(StyleClass::OverflowYAuto);
        }
        style
    });
    let unstyled_scroll = explicit_scroll && style.is_none();
    let content = match layout {
        RederivedLayout::Grid { cols, gap } => View::Grid {
            cols,
            gap,
            cells: children,
            style,
        },
        RederivedLayout::Row | RederivedLayout::Column => {
            let builder = match layout {
                RederivedLayout::Row => View::row(),
                _ => View::col(),
            };
            let builder = if let Some(style) = style {
                builder.with_style(style)
            } else {
                builder
            };
            builder.children(children).build()
        }
    };
    if unstyled_scroll {
        View::scrollable(content).build()
    } else {
        content.with_overflow_scroll()
    }
}

fn pointer_events(
    content: View<NodeDesktopMsg>,
    on_click: Option<NodeDesktopMsg>,
    on_double_click: Option<NodeDesktopMsg>,
    on_context_menu: Option<NodeDesktopMsg>,
) -> View<NodeDesktopMsg> {
    if on_click.is_none() && on_double_click.is_none() && on_context_menu.is_none() {
        return content;
    }
    View::MouseArea {
        content: Box::new(content),
        on_click,
        on_enter: None,
        on_exit: None,
        on_double_click,
        on_context_menu,
        on_release: None,
        on_move: None,
        logical_extent: None,
        style: None,
    }
}

fn first_text(items: &[WidgetViewItem]) -> Option<&str> {
    items.iter().find_map(|item| match item {
        WidgetViewItem::Text(text) => Some(text.as_str()),
        WidgetViewItem::Element(element) => element.label.as_deref()
            .or_else(|| prop(element, "text"))
            .or_else(|| first_text(&element.children)),
        WidgetViewItem::Timer(_) => None,
    })
}

fn prop<'a>(element: &'a WidgetElement, key: &str) -> Option<&'a str> {
    element
        .properties
        .iter()
        .rev()
        .find_map(|(name, value)| (name == key).then_some(value.as_str()))
}

fn style(element: &WidgetElement) -> Option<Style> {
    let mut properties = ["style", "class"]
        .into_iter()
        .filter_map(|key| prop(element, key));
    let first = properties.next();
    let second = properties.next();
    match (first, second) {
        (Some(first), Some(second)) => Style::parse(&format!("{first} {second}")).ok(),
        (Some(one), None) => Style::parse(one).ok(),
        _ => {
            let default = match element.tag.as_str() {
                "h1" => "text-2xl font-semibold",
                "h2" => "text-lg font-semibold",
                "preview-card" => "rounded-md border p-4",
                "codeblock" => "font-mono bg-muted p-2",
                "alert-dialog-content" => {
                    "w-96 bg-background border border-border rounded-lg shadow-lg p-6 gap-4"
                }
                "alert-dialog-title" => "text-lg font-semibold",
                "alert-dialog-description" => "text-sm text-muted-foreground",
                "alert-dialog-header" => "gap-2",
                "alert-dialog-footer" => "justify-end gap-2",
                "alert-dialog-cancel" => "border bg-background text-foreground rounded-md px-4 py-2",
                "alert-dialog-action" => "bg-primary text-primary-foreground rounded-md px-4 py-2",
                _ => return None,
            };
            Style::parse(default).ok()
        }
    }
}
