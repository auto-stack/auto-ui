use auto_widgets::*;
use auto_gui::*;
use autoval::AutoStr;
use std::rc::Rc;
use std::cell::RefCell;

// when using gpui as base
use auto_adapter::*;
pub struct AutoApp {
    widget: Text,
    base: AutoGuiApp,
    adapter: Rc<RefCell<AutoAdapterImpl>>,
}

impl AutoApp {
    pub fn new(text: impl Into<AutoStr>) -> Self {
        let widget = Text { text: text.into() };
        let base = AutoGuiApp::center();
        let adapter = Rc::new(RefCell::new(AutoAdapterImpl::new()));
        Self { widget, base, adapter }
    }

    pub fn run(&self) {
        let text = self.widget.clone();
        let adapter = self.adapter.clone();
        self.base.run(move |_| adapter.borrow().text(text.clone()));
    }
}




