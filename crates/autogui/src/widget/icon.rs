use crate::style::color::black;
use gpui::*;

#[derive(IntoElement)]
pub struct Icon {
    base: Svg,
    path: SharedString,
    color: Hsla,
    size: Rems,
}

impl Icon {
    pub fn new(path: SharedString) -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path,
            color: black(),
            size: Rems(1.),
        }
    }

    pub fn size(mut self, size: Rems) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }

    pub fn transform(mut self, transformation: gpui::Transformation) -> Self {
        self.base = self.base.with_transformation(transformation);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        self.base
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
    Loader,
    ArrowUp,
    ArrowDown,
    Reload,
}

impl SysIcon {
    pub fn icon(self) -> Icon {
        Icon::from(self)
    }
}

impl From<SysIcon> for Icon {
    fn from(value: SysIcon) -> Self {
        match value {
            SysIcon::Check => Icon::new("icons/check.svg".into()),
            SysIcon::Sun => Icon::new("icons/sun.svg".into()),
            SysIcon::Moon => Icon::new("icons/moon.svg".into()),
            SysIcon::Loader => Icon::new("icons/loader.svg".into()),
            SysIcon::ArrowUp => Icon::new("icons/arrow_up.svg".into()),
            SysIcon::ArrowDown => Icon::new("icons/arrow_down.svg".into()),
            SysIcon::Reload => Icon::new("icons/reload.svg".into()),
        }
    }
}
