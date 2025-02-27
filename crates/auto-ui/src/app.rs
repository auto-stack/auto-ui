use auto_widgets::*;
use auto_gui::*;
use autoval::AutoStr;
use std::rc::Rc;
use std::cell::RefCell;

// when using gpui as base
use auto_adapter::*;
pub struct AutoApp {
    pane: Pane,
    base: AutoGuiApp,
    adapter: Rc<RefCell<AutoAdapterImpl>>,
}

impl AutoApp {
    pub fn new() -> Self {
        let pane = Pane::Empty;
        let base = AutoGuiApp::new();
        let adapter = Rc::new(RefCell::new(AutoAdapterImpl::new()));
        Self { pane, base, adapter }
    }

    pub fn center(mut self, texts: Vec<Text>) -> Self {
        self.pane = Pane::Center(texts.into_iter().map(|text| Kid::Widget(Rc::new(RefCell::new(text)))).collect());
        self
    }

    pub fn run(&self) {
        let pane = self.pane.clone();
        let adapter = self.adapter.clone();
        self.base.run(move |cx| adapter.borrow().pane(pane.clone(), cx));
    }
}




