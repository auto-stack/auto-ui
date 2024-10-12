use gpui::*;
use crate::theme::color::black;

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    color: Hsla,
    size: Rems,
}

impl Icon {
    pub fn new(path: SharedString) -> Self {
        Self { path, color: black(), size: Rems(1.) }
    }

    pub fn size(mut self, size: Rems) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        svg()
            .size(self.size)
            .flex_none()
            .path(self.path)
            .text_color(self.color)
    }
}

pub enum SysIcon {
    Check,
    Sun,
    Moon,
}

impl From<SysIcon> for Icon {
    fn from(value: SysIcon) -> Self {
        match value {
            SysIcon::Check => Icon::new("icons/check.svg".into()),
            SysIcon::Sun => Icon::new("icons/sun.svg".into()),
            SysIcon::Moon => Icon::new("icons/moon.svg".into()),
        }
    }
}

