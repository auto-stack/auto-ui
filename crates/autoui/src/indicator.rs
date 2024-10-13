use std::time::Duration;

use crate::widget::icon::{Icon, SysIcon};
use crate::style::size::{icon_size_for, Sizer, SizeScale};
use gpui::Size;
use gpui::{
    div, ease_in_out, percentage, prelude::FluentBuilder as _, Animation, AnimationExt as _, Hsla,
    IntoElement, ParentElement, RenderOnce, Styled as _, Transformation, WindowContext,
};

#[derive(IntoElement)]
pub struct Indicator {
    size: SizeScale,
    icon: Icon,
    speed: Duration,
    color: Option<Hsla>,
}

impl Indicator {
    pub fn new() -> Self {
        Self {
            size: SizeScale::M,
            speed: Duration::from_secs_f64(0.8),
            icon: SysIcon::Loader.icon(),
            color: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn size(mut self, size: impl Into<SizeScale>) -> Self {
        self.size = size.into();
        self
    }

    pub fn size_scale(mut self, size_scale: SizeScale) -> Self {
        self.size = size_scale;
        self
    }
}

impl RenderOnce for Indicator {
    fn render(self, _: &mut WindowContext) -> impl IntoElement {
        div()
            .child(
                self.icon
                    .size(icon_size_for(self.size))
                    .when_some(self.color, |this, color| this.color(color))
                    .with_animation(
                        "circle",
                        Animation::new(self.speed).repeat().with_easing(ease_in_out),
                        |this, delta| this.transform(Transformation::rotate(percentage(delta))),
                    ),
            )
            .into_element()
    }
}
