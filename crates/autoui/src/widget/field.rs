use gpui::*;

pub struct Field {
    label: SharedString,
    input: Div,
}

impl Field {
    pub fn new(label: &str) -> Self {
        Self { label: SharedString::from(label.to_string()), input: div() }
    }
}