use gpui::*;
use gpui::prelude::*;
use crate::style::theme::ActiveTheme;
use std::{cell::Cell, rc::Rc};
use crate::widget::Axis;

#[derive(IntoElement)]
pub struct Radio {
    pub id: ElementId,
    pub selected: bool,
    pub label: String,
    on_click: Option<Box<dyn Fn(&bool, &mut WindowContext) + 'static>>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            selected: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(mut self, on_click: impl Fn(&bool, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn on_click_mut<T: Render>(mut self, cx: &mut ViewContext<T>, handler: impl Fn(&mut T, &bool, &mut ViewContext<'_, T>) + 'static) -> Self {
        self.on_click = Some(Box::new(cx.listener(move |view, selected, cx| {
            (handler)(view, selected, cx);
        })));
        self
    }
}

impl RenderOnce for Radio {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            // styles
            .flex()
            .flex_row()
            .items_center()
            .id(self.id)
            .gap_x_2()
            .cursor_pointer()
            .text_color(theme.foreground)
            .line_height(relative(1.0))
            // radio circle
            .child(
                div()
                    .relative()
                    .size_4()
                    .flex_shrink_0()
                    .rounded_full()
                    .border_1()
                    .border_color(theme.primary)
                    .when(self.selected, |this| this.bg(theme.primary))
                    .child(
                        svg()
                            .absolute()
                            .top_px()
                            .left_px()
                            .size_3()
                            .text_color(theme.primary)
                            .when(self.selected, |this| {
                                this
                                    .text_color(theme.primary_foreground)
                                    .path("icons/check.svg")
                            })
                    )
            )
            // label
            .when(!self.label.is_empty(), |this| this.child(
                self.label
            ))
            // click
            .on_click(move |_ev, cx| {
                if let Some(handler) = &self.on_click {
                    handler(&true, cx);
                }
            })
    }
}

#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    selected: Option<usize>,
    radios: Vec<Radio>,
    axis: Axis,
    on_click: Option<Box<dyn Fn(&usize, &mut WindowContext) + 'static>>,
}

impl RadioGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            radios: Vec::new(),
            on_click: None,
            axis: Axis::Horiz,
        }
    }

    pub fn add(mut self, radio: Radio) -> Self {
        self.radios.push(radio);
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn select(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn on_click(mut self, on_click: impl Fn(&usize, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let state = Rc::new(Cell::new(None));
        div()
            .id(self.id)
            .flex()
            .map(|this| {
                match self.axis {
                    Axis::Horiz => this.flex_row().gap_x_4(),
                    Axis::Vert => this.flex_col().gap_y_4(),
                }
            })
            .children(
                self.radios.into_iter().enumerate().map(|(i, radio)| {
                    let state = Rc::clone(&state);
                    radio
                        .selected(self.selected.map(|s| s == i).unwrap_or(false))
                        .on_click(move |_, _| {
                            println!("Selected: {}", i);
                            state.set(Some(i));
                        })
                })
            )
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |_, cx| {
                    handler(&state.get().unwrap_or(0), cx);
                })
            })
    }
}
