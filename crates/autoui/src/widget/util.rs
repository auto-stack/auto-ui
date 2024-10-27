use crate::style::theme::ActiveTheme;
use gpui::*;

pub fn card(title: impl IntoElement, cx: &WindowContext) -> Div {
    let theme = cx.active_theme();
    div()
        .flex()
        .flex_col()
        .items_start()
        .justify_between()
        .p_4()
        .w_full()
        .bg(theme.card)
        .border_1()
        .border_color(theme.border)
        // Title
        .child(
            div()
                .flex_none()
                .w_full()
                .font_weight(FontWeight::BOLD)
                .text_lg()
                .border_b_1()
                .border_color(theme.border)
                .child(title),
        )
        .gap_4()
}

// pub fn field()

pub fn col() -> Div {
    div().flex().flex_col()
}

pub fn row() -> Div {
    div().flex().flex_row()
}

pub fn field(label: &str, input: impl IntoElement) -> Div {
    row().w_full().items_center()
        .child(div().min_w(px(100.0)).child(SharedString::from(label.to_string())))
        .child(div().flex().flex_row().w_full().max_w(px(200.)).child(input))
}

