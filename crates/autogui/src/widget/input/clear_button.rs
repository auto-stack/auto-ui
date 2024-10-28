use gpui::{px, WindowContext};
use crate::style::size::SizeScale;
use crate::widget::button::Button;
use crate::widget::icon::SysIcon;

pub(crate) struct ClearButton {}

impl ClearButton {
    pub fn new(_: &mut WindowContext) -> Button {
        Button::iconed(SysIcon::Moon.icon())
            .size_scale(SizeScale::S)
    }
}
