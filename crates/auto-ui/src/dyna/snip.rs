use auto_val::AutoStr;

#[derive(Clone, Debug)]
pub struct Snip {
    pub code: AutoStr,
}

impl Snip {
    pub fn new(code: impl Into<AutoStr>) -> Self {
        Self { code: code.into() }
    }
}
