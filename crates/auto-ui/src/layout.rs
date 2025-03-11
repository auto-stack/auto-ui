use gpui::{div, Div, Styled};
use gpui_component::gray_200;

#[inline]
pub fn center() -> Div {
    div().flex().flex_col().size_full().items_center().justify_center()
}