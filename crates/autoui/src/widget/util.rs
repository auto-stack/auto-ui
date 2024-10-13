use crate::style::theme::ActiveTheme;
use gpui::*;

pub fn card(title: impl IntoElement, cx: &WindowContext) -> Div {
    let theme = cx.active_theme();
    div()
        .flex()
        .flex_col()
        .items_start()
        .p_4()
        .w_full()
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
        .child(
            div()
                .flex_none()
                .w_full()
                .font_weight(FontWeight::BOLD)
                .text_xl()
                .child(title),
        )
        .gap_4()
}

pub fn col() -> Div {
    div().flex().flex_col()
}

pub fn row() -> Div {
    div().flex().flex_row()
}
