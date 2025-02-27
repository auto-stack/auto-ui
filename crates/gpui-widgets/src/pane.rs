use gpui::*;
use crate::TextView;

pub enum PaneView {
    Empty,
    // Raw(Kids),
    Center(Kids),
    // Left(Kids),
    // Right(Kids),
    // Bottom(Kids),
    // Top(Kids),
    // Row(Kids),
    // Col(Kids),
}

pub type Kids = Vec<KidView>;

pub enum KidView {
    // Pane(PaneView),
    View(AnyView),
}

impl PaneView {
    pub fn new(views: Vec<impl Into<AnyView>>) -> Self {
        PaneView::Center(views.into_iter().map(|view| KidView::View(view.into())).collect())
    }
}

impl Render for PaneView {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self {
            PaneView::Center(kids) => {
                let views: Vec<AnyElement> = kids.iter().map(|kid| match kid {
                    KidView::View(view) => view.clone().into_any_element(),
                    // _ => div().into_any_element()
                }).collect();
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
                            .children(views)
                    )
            }
            PaneView::Empty => {
                div().size_full().child("Empty Pane")
            }
        }
    }
}
