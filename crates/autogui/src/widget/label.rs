use gpui::*;

#[derive(IntoElement)]
pub struct Label {
    pub text: SharedString,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for Label {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div().child(self.text)
    }
}

