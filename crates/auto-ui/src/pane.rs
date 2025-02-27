use auto_gui::*;
use auto_widgets::*;

pub struct Pane {
    widgets: Vec<Text>,
}

impl Pane {
    pub fn new(widgets: Vec<Text>) -> Self {
        Self { widgets }
    }

}

