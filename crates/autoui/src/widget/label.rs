use gpui::*;

#[derive(IntoElement)]
pub struct Label {
    pub text: SharedString,
}

impl RenderOnce for Label {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div().child(self.text.clone())
    }
}    

