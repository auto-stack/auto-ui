use gpui::*;

pub struct Center {
    pub child: AnyView,
}

impl Center {
    pub fn new(child: impl Into<AnyView>) -> Self {
        Self { child: child.into() }
    }
}

impl Render for Center {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                    .child(self.child.clone()))
    }
}

