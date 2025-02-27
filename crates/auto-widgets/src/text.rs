use autoval::AutoStr;
use crate::Widget;

#[derive(Clone)]
pub struct Text {
    pub text: AutoStr,
}

impl Text {
    pub fn new(text: impl Into<AutoStr>) -> Self {
        Self { text: text.into() }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self { text: AutoStr::new() }
    }
}

impl Widget for Text {
    fn id(&self) -> AutoStr {
        self.text.clone()
    }
}

