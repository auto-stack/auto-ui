use std::ops::Deref;

use gpui::*;
use crate::TextView;
use crate::ButtonView;
use auto_widgets::*;
use autoval::AutoStr;
pub struct PaneView {
    pane: Pane,
}

impl PaneView {
    // pub fn new(views: Vec<impl Into<AnyView>>) -> Self {
        // PaneView::Center(views.into_iter().map(|view| KidView::View(view.into())).collect())
    // }
    pub fn new(pane: Pane) -> Self {
        Self { pane }
    }
}

impl Render for PaneView {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let kids = self.make_views(_w, _cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .children(kids)
            )
    }
}

impl PaneView {
    fn make_views(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut pane = std::mem::take(&mut self.pane);
        let views: Vec<AnyElement> = match &mut pane {
            Pane::Empty => vec![],
            Pane::Center(kids) => kids.iter().map(|kid| {
                match kid {
                    Kid::Widget(widget) => {
                        match widget {
                            Widget::Text(text) => {
                                let text = text.text.clone();
                                cx.new(|_| TextView::new(text)).into_any_element()
                            }
                            Widget::Button(button) => {
                                let text = button.text.clone();
                                let btext = text.clone();
                                ButtonView::primary(text)
                                    .on_click(move |_ev, _w, _cx| {
                                        println!("Button clicked: {}", btext.clone());
                                    })
                                    .on_click_mut(_w, cx, |v, e, w, cx| {
                                        match &mut v.pane {
                                            Pane::Empty => {
                                                v.pane = Pane::Center(vec![Kid::Widget(Widget::Text(Text::new("Hello, World!")))])
                                            }
                                            Pane::Center(kids) => {
                                                for kid in kids.iter_mut() {
                                                    if let Kid::Widget(widget) = kid {
                                                        if let Widget::Text(text) = widget {
                                                            text.text = AutoStr::from("Hello, new World!");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    })
                                    .into_any_element()
                            }
                        }
                    }
                }
            }).collect(),
        };
        self.pane = pane;
        views
    }
}
