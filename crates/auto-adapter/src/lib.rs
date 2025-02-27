use auto_widgets::*;

// when using gpui as base
use gpui::*;
use gpui_widgets::*;

pub type TextWidget = gpui_widgets::TextView;
pub type AutoAdapterImpl = GpuiAdapter;

pub trait AutoAdapter {
    fn text(&self, text: Text) -> TextWidget;
}

pub struct GpuiAdapter;

impl GpuiAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl AutoAdapter for GpuiAdapter {
    fn text(&self, text: Text) -> TextWidget {
        TextWidget::new(text.text.clone())
    }
}