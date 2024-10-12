use gpui::prelude::*;
use gpui::*;
use smallvec::SmallVec;
use crate::style::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Card {
    title: SharedString,
    children: SmallVec<[AnyElement; 4]>,
}

impl Card {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { title: title.into(), children: SmallVec::new() }
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Card {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.active_theme();

        div()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .p_2()
            .w_full()
            .bg(theme.card)
            .border_1()
            .border_color(theme.border)
            .child(div().flex_none().w_full().child(self.title))
            .children(self.children)
    }
}
