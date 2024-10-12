use gpui::*;
use gpui::prelude::*;
use crate::style::theme::ActiveTheme;

pub struct Slider {
    id: ElementId,
    min: f32,
    max: f32,
    value: f32,
    bounds: Bounds<Pixels>,
}

pub enum SliderEvent {
    Move(f32),
}

#[derive(Clone, Render)]
pub struct DragThumb(EntityId);


impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            min: 0.0,
            max: 100.0,
            value: 0.0,
            bounds: Bounds::default(),
        }
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn initial_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn update_value(&mut self, value: f32, cx: &mut ViewContext<Self>) {
        self.value = value;
        cx.notify();
    }

    pub fn relative_value(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }

    pub fn render_thumb(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let value = self.value;
        let entity_id = cx.entity_id();

        div()
            .id("slider-thumb")
            .on_drag(DragThumb(entity_id), |drag, cx| {
                cx.stop_propagation();
                cx.new_view(|_| drag.clone())
            })
            .on_drag_move(cx.listener(
                move |view, ev: &DragMoveEvent<DragThumb>, cx| match ev.drag(cx) {
                    DragThumb(id) => {
                        if *id != entity_id {
                            return;
                        }

                        view.update_value_by_position(ev.event.position, cx);
                    }
                }
            ))
            .absolute()
            .top(px(-5.))
            .left(relative(self.relative_value()))
            .ml(-px(8.))
            .size_4()
            .rounded_full()
            .border_1()
            .border_color(cx.active_theme().slider_bar.opacity(0.9))
            .bg(cx.active_theme().slider_thumb)
    }

    pub fn update_value_by_position(&mut self, position: Point<Pixels>, cx: &mut ViewContext<Self>) {
        let bounds = self.bounds;
        let min = self.min;
        let max = self.max;

        let value = {
            let relative = (position.x - bounds.left()) / bounds.size.width;
            min + (max - min) * relative
        };

        self.value = value.clamp(min, max);
        cx.emit(SliderEvent::Move(self.value));
        cx.notify();
    }

    pub fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.update_value_by_position(event.position, cx);
    }
    
}

impl EventEmitter<SliderEvent> for Slider {}

impl Render for Slider {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .id(self.id.clone())
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .h_5()
            .child(
                // slider bar
                div()
                    .id("slider-bar")
                    .relative()
                    .w_full()
                    .my_1p5()
                    .h_1p5()
                    .bg(theme.slider_bar.opacity(0.2))
                    .active(|this| this.bg(theme.slider_bar.opacity(0.4)))
                    .rounded(px(3.))
                    // slider track
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .h_full()
                            .w(relative(self.relative_value()))
                            .bg(theme.slider_bar)
                            .rounded_l(px(3.))
                    )
                    // thumb
                    .child(self.render_thumb(cx))
                    // canvas used for rendering draggable thumb. TODO: figure out how it works
                    .child({
                        let view = cx.view().clone();
                        canvas(
                            move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                            |_,_,_| {},
                        )
                        .absolute()
                        .size_full()
                    })
            )
    }
}