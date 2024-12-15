use gpui::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;
use crate::style::theme::ActiveTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Vertical,
    Horizontal,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarState {
    hover_axis: Option<ScrollbarAxis>,
    hover_on_thumb: Option<ScrollbarAxis>,
    dragged_axis: Option<ScrollbarAxis>,
    drag_pos: Point<Pixels>,
    last_scroll_offset: Point<Pixels>,
    last_scroll_time: Option<Instant>,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            hover_axis: None,
            hover_on_thumb: None,
            dragged_axis: None,
            drag_pos: point(px(0.0), px(0.0)),
            last_scroll_offset: point(px(0.0), px(0.0)),
            last_scroll_time: None,
        }
    }
}

impl ScrollbarState {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_drag_pos(&self, axis: ScrollbarAxis, pos: Point<Pixels>) -> Self {
        let mut state = *self;
        state.drag_pos.y = pos.y;
        state.dragged_axis = Some(axis);
        state
    }

    fn with_unset_drag_pos(&self) -> Self {
        let mut state = *self;
        state.dragged_axis = None;
        state
    }

    fn with_hovered(&self, axis: Option<ScrollbarAxis>) -> Self {
        let mut state = *self;
        state.hover_axis = axis;
        state
    }

    fn with_hovered_on_thumb(&self, axis: Option<ScrollbarAxis>) -> Self {
        let mut state = *self;
        state.hover_on_thumb = axis;
        state
    }

    pub fn with_last_scroll(&self, offset: Point<Pixels>, time: Option<Instant>) -> Self {
        let mut state = *self;
        state.last_scroll_offset = offset;
        state.last_scroll_time = time;
        state
    }
}

pub trait ScrollHandleOffsetable {
    fn offset(&self) -> Point<Pixels>;
    fn set_offset(&self, offset: Point<Pixels>);
    fn is_uniform_list(&self) -> bool {
        false
    }
}

impl ScrollHandleOffsetable for UniformListScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.0.borrow().base_handle.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.0.borrow_mut().base_handle.set_offset(offset)
    }

    fn is_uniform_list(&self) -> bool {
        true
    }
}

pub struct Scrollbar {
    view_id: EntityId,
    axis: ScrollbarAxis,
    width: Pixels,
    scroll_handle: Rc<Box<dyn ScrollHandleOffsetable>>,
    scroll_size: Size<Pixels>,
    state: Rc<Cell<ScrollbarState>>,
}

impl Scrollbar {
    pub fn new(
        view_id: EntityId,
        axis: ScrollbarAxis,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: Size<Pixels>,
        state: Rc<Cell<ScrollbarState>>,
    ) -> Self {
        Self {
            view_id,
            axis,
            width: px(12.0),
            scroll_handle: Rc::new(Box::new(scroll_handle)),
            scroll_size,
            state,
        }
    }

    pub fn vertical(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: Size<Pixels>,
    ) -> Self {
        Self::new(view_id, ScrollbarAxis::Vertical, scroll_handle, scroll_size, state)
    }

    pub fn uniform_scroll(view_id: EntityId, state: Rc<Cell<ScrollbarState>>, scroll_handle: UniformListScrollHandle) -> Self {
        let scroll_size = scroll_handle.0.borrow().last_item_size.map(|size| size.contents).unwrap_or_default();

        Self::new(view_id, ScrollbarAxis::Vertical, scroll_handle, scroll_size, state)
    }
}


impl IntoElement for Scrollbar {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

const MIN_THUMB_SIZE: Pixels = px(80.0);
const THUMB_RADIUS: Pixels = Pixels(3.0);
const THUMB_INSET: Pixels = Pixels(4.);

impl Element for Scrollbar {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, cx: &mut WindowContext) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.position = Position::Absolute;
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();

        (cx.request_layout(style, None), ())
    }

    fn prepaint(&mut self,
        _: Option<&GlobalElementId>, 
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            cx.insert_hitbox(bounds, false)
        })
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let hitbox_bounds = hitbox.bounds;
        
        cx.with_content_mask(Some(ContentMask { bounds: hitbox_bounds }), |cx| {
            let (scroll_area_size, container_size, scroll_position) = (
                self.scroll_size.height,
                hitbox_bounds.size.height,
                self.scroll_handle.offset().y,
            );

            let margin_end = px(0.0);

            const NORMAL_OPACITY: f32 = 0.35;

            let thumb_length = (container_size / scroll_area_size * container_size).max(MIN_THUMB_SIZE);
            let thumb_start = -(scroll_position / (scroll_area_size - container_size) * (container_size - margin_end - thumb_length));
            let thumb_end = (thumb_start + thumb_length).min(container_size - margin_end);

            let bounds = Bounds {
                origin: point(
                    hitbox_bounds.origin.x + hitbox_bounds.size.width - self.width,
                    hitbox_bounds.origin.y,
                ),
                size: Size {
                    width: self.width,
                    height: hitbox_bounds.size.height,
                },
            };

            let state = self.state.clone();
            let theme = cx.active_theme();
            let (thumb_bg, bar_bg, bar_border, inset, radius) = if state.get().dragged_axis
                        == Some(self.axis)
                    {
                        (
                            theme.scrollbar_thumb,
                            theme.scrollbar,
                            theme.border,
                            THUMB_INSET - px(1.),
                            THUMB_RADIUS,
                        )
                    } else if state.get().hover_axis == Some(self.axis) {
                        if state.get().hover_on_thumb == Some(self.axis) {
                            (
                                theme.scrollbar_thumb,
                                theme.scrollbar,
                                theme.border,
                                THUMB_INSET - px(1.),
                                THUMB_RADIUS,
                            )
                        } else {
                            (
                                theme.scrollbar_thumb.opacity(NORMAL_OPACITY),
                                transparent_black(),
                                transparent_black(),
                                THUMB_INSET,
                                THUMB_RADIUS,
                            )
                        }
                    } else {
                        let mut idle_state = (
                            gpui::transparent_black(),
                            gpui::transparent_black(),
                            gpui::transparent_black(),
                            THUMB_INSET,
                            THUMB_RADIUS - px(1.),
                        );
                        if let Some(last_time) = state.get().last_scroll_time {
                            let elapsed = Instant::now().duration_since(last_time).as_secs_f32();
                            if elapsed < 1.0 {
                                let y_value = NORMAL_OPACITY - elapsed.powi(10); // y = 1 - x^10
                                idle_state.0 = theme.scrollbar_thumb.opacity(y_value);
                                cx.request_animation_frame();
                            }
                        }
                        idle_state
                    };

                    let border_width = px(0.);
                    let thumb_bounds = Bounds::from_corners(
                        point(
                            bounds.origin.x + inset + border_width,
                            bounds.origin.y + thumb_start + inset,
                        ),
                        point(
                            bounds.origin.x + self.width - inset,
                            bounds.origin.y + thumb_end - inset,
                        ),
                    );

                    cx.paint_quad(fill(bounds, bar_bg));

                    cx.paint_quad(PaintQuad {
                        bounds,
                        corner_radii: (0.).into(),
                        background: gpui::transparent_black().into(),
                        border_widths: Edges {
                            top: px(0.),
                            right: px(0.),
                            bottom: px(0.),
                            left: border_width,
                        },
                        border_color: bar_border,
                    });

                    cx.paint_quad(fill(thumb_bounds, thumb_bg).corner_radii(radius));

                    cx.on_mouse_event({
                        let state = self.state.clone();
                        let view_id = self.view_id;
                        let scroll_handle = self.scroll_handle.clone();

                        move |event: &ScrollWheelEvent, phase, cx| {
                            if phase.bubble() && hitbox_bounds.contains(&event.position) {
                                if scroll_handle.offset() != state.get().last_scroll_offset {
                                    state.set(state.get().with_last_scroll(
                                        scroll_handle.offset(),
                                        Some(Instant::now()),
                                    ));
                                    cx.notify(Some(view_id));
                                }
                            }
                        }
                    });

                    cx.on_mouse_event({
                        let state = self.state.clone();
                        let view_id = self.view_id;
                        let scroll_handle = self.scroll_handle.clone();
                        let axis = self.axis;

                        move |event: &MouseDownEvent, phase, cx| {
                            if phase.bubble() && bounds.contains(&event.position) {
                                cx.stop_propagation();

                                if thumb_bounds.contains(&event.position) {
                                    // click on the thumb bar, set the drag position
                                    let pos = event.position - thumb_bounds.origin;

                                    state.set(state.get().with_drag_pos(axis, pos));

                                    cx.notify(Some(view_id));
                                } else {
                                    // click on the scrollbar, jump to the position
                                    // Set the thumb bar center to the click position
                                    let offset = scroll_handle.offset();
                                    let percentage = ((event.position.y - thumb_length / 2. - bounds.origin.y)
                                            / (bounds.size.height - thumb_length)).min(1.);

                                    scroll_handle.set_offset(point(
                                        offset.x,
                                        -scroll_area_size * percentage,
                                    ));
                                }
                            }
                        }
                    });

                    cx.on_mouse_event({
                        let scroll_handle = self.scroll_handle.clone();
                        let state = self.state.clone();
                        let view_id = self.view_id;
                        let axis = self.axis;
                        move |event: &MouseMoveEvent, _, cx| {
                            // Update hovered state for scrollbar
                            if bounds.contains(&event.position) {
                                if state.get().hover_axis != Some(axis) {
                                    state.set(state.get().with_hovered(Some(axis)));
                                    cx.notify(Some(view_id));
                                }
                            } else {
                                if state.get().hover_axis == Some(axis) {
                                    if state.get().hover_axis.is_some() {
                                        state.set(state.get().with_hovered(None));
                                        cx.notify(Some(view_id));
                                    }
                                }
                            }

                            // Update hovered state for scrollbar thumb
                            if thumb_bounds.contains(&event.position) {
                                if state.get().hover_on_thumb != Some(axis) {
                                    state.set(state.get().with_hovered_on_thumb(Some(axis)));
                                    cx.notify(Some(view_id));
                                }
                            } else {
                                if state.get().hover_on_thumb == Some(axis) {
                                    state.set(state.get().with_hovered_on_thumb(None));
                                    cx.notify(Some(view_id));
                                }
                            }

                            // Move thumb position on dragging
                            if state.get().dragged_axis == Some(axis) && event.dragging() {
                                // drag_pos is the position of the mouse down event
                                // We need to keep the thumb bar still at the origin down position
                                let drag_pos = state.get().drag_pos;

                                let percentage = ((event.position.y - drag_pos.y - bounds.origin.y) / (bounds.size.height - thumb_length)).clamp(0., 1.);

                                let offset = point(
                                    scroll_handle.offset().x,
                                    -(scroll_area_size - container_size) * percentage,
                                );

                                scroll_handle.set_offset(offset);
                                cx.notify(Some(view_id));
                            }
                        }
                    });

                    cx.on_mouse_event({
                        let view_id = self.view_id;
                        let state = self.state.clone();

                        move |_event: &MouseUpEvent, phase, cx| {
                            if phase.bubble() {
                                state.set(state.get().with_unset_drag_pos());
                                cx.notify(Some(view_id));
                            }
                        }
                    });

        })
    }
}
