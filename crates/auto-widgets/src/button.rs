use autoval::AutoStr;

#[derive(Clone)]
pub struct Button {
    pub text: AutoStr,
}

impl Button {
    pub fn new(text: impl Into<AutoStr>) -> Self {
        Self { text: text.into() }
    }
}

impl Button {
    pub fn id(&self) -> AutoStr {
        self.text.clone()
    }
}
