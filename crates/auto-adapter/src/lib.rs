use auto_widgets::*;
use std::rc::Rc;
use std::cell::RefCell;

// when using gpui as base
use gpui::*;
use gpui_widgets::*;

pub type TextImpl = gpui_widgets::TextView;
pub type PaneImpl = gpui_widgets::PaneView;
pub type AutoAdapterImpl = GpuiAdapter;

pub trait AutoAdapter {
    fn text(&self, text: Rc<RefCell<Text>>, cx: &mut Context<TextImpl>) -> TextImpl;
    fn pane(&self, pane: Pane, cx: &mut Context<PaneImpl>) -> PaneImpl;
}

pub struct GpuiAdapter;

impl GpuiAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl AutoAdapter for GpuiAdapter {
    fn text(&self, text: Rc<RefCell<Text>>, _cx: &mut Context<TextImpl>) -> TextImpl {
        TextImpl::new(text.borrow().text.clone())
    }

    fn pane(&self, pane: Pane, cx: &mut Context<PaneImpl>) -> PaneImpl {
        match pane {    
            Pane::Center(kids) => {
                let views: Vec<KidView> = kids.iter().map(|kid| match kid {
                    Kid::Widget(text) => {
                        KidView::View(cx.new(|cx| self.text(text.clone(), cx)).into())
                    }
                }).collect();
                PaneImpl::Center(views)
            }
            Pane::Empty => {
                PaneImpl::Empty
            }
        }
    }
}